use std::env;
use std::fmt::Display;
use std::path::PathBuf;
use std::process::Stdio;

use log::{debug, info};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;

use crate::TorrentExtra;
use crate::api_types::TorrentInfo;
use crate::qbittorrent_web_api::{
    QBittorrentWebApiResult, add_torrent, get_torrent_list, remove_torrent, set_torrent_category,
};

#[derive(Debug)]
pub struct QBittorrentClient {
    pub profile_dir: PathBuf,
}

#[derive(Debug)]
pub(crate) struct QBittorrentClientProcess {
    pub process_handle: JoinHandle<()>,
    pub port: usize,
}

const QBITTORRENT_INI_FILE_CONTENTS: &str = r#"[LegalNotice]
Accepted=true

[Preferences]
General\Locale=en
MailNotification\req_auth=true
WebUI\Address=127.0.0.1
WebUI\LocalHostAuth=false
WebUI\Port=0
"#;

impl QBittorrentClient {
    /// `None` as `profile_dir` will use a temp directory. Ideal for testing.
    pub fn try_new(profile_dir: Option<PathBuf>) -> QBittorrentResult<Self> {
        Ok(Self {
            profile_dir: profile_dir.unwrap_or(env::temp_dir().join("streamy-qbittorrent")),
        })
    }

    pub async fn event_loop(
        &self,
        mut receiver: tokio::sync::mpsc::Receiver<QBittorrentClientMessage>,
        state_updater: tokio::sync::watch::Sender<Box<[TorrentInfo]>>,
    ) -> QBittorrentResult<()> {
        let mut process_client = Some(self.spawn_qbittorrent_web().await?);
        let http_client = reqwest::Client::new();

        loop {
            let message = if let Some(message) = receiver.recv().await {
                message
            } else {
                break;
            };

            match message {
                QBittorrentClientMessage::SetExtra {
                    id,
                    extra,
                    result_sender,
                } => {
                    let process_client = if let Some(client) = &process_client {
                        client
                    } else {
                        debug!("Spawning QBittorrent to set a category");
                        process_client = Some(self.spawn_qbittorrent_web().await?);
                        process_client.as_ref().unwrap()
                    };

                    let result =
                        set_torrent_category(&http_client, process_client.port, &id, &extra).await;
                    // TODO: add logging here
                    let _ = result_sender.send(result);
                }
                QBittorrentClientMessage::AddTorrent {
                    hash,
                    result_sender,
                    extra,
                } => {
                    let process_client = if let Some(client) = &process_client {
                        client
                    } else {
                        debug!("Spawning QBittorrent to add a new torrent");
                        process_client = Some(self.spawn_qbittorrent_web().await?);
                        process_client.as_ref().unwrap()
                    };

                    let result =
                        add_torrent(&http_client, process_client.port, &hash, &extra).await;
                    // TODO: add logging here
                    let _ = result_sender.send(result);
                }
                QBittorrentClientMessage::RemoveTorrent { id, result_sender } => {
                    let process_client = if let Some(client) = &process_client {
                        client
                    } else {
                        debug!("Spawning QBittorrent to remove a torrent");
                        process_client = Some(self.spawn_qbittorrent_web().await?);
                        process_client.as_ref().unwrap()
                    };

                    let result = remove_torrent(&http_client, process_client.port, &id).await;
                    // TODO: add logging here
                    let _ = result_sender.send(result);
                }
                QBittorrentClientMessage::UpdateTorrentList { result_sender } => {
                    let process_client_ref = if let Some(process_client) = &process_client {
                        process_client
                    } else {
                        debug!("QBittorrent client is down. Not going to update the torrent list.");
                        let _ = result_sender.send(Ok(()));
                        continue;
                    };

                    match get_torrent_list(&http_client, process_client_ref.port).await {
                        Ok(torrent_list) => {
                            // If all torrents are done, drop the client.
                            if torrent_list.is_empty()
                                || torrent_list.iter().all(|item| item.state.should_stop())
                            {
                                debug!(
                                    "No torrents are being downloaded. Killing the QBittorrent process"
                                );
                                process_client = None;
                            }

                            // TODO: add logging here
                            let _ = state_updater.send(torrent_list);
                            let _ = result_sender.send(Ok(()));
                        }
                        Err(err) => {
                            // TODO: add logging here
                            let _ = result_sender.send(Err(err));
                        }
                    }
                }
            };
        }

        Ok(())
    }

    pub(crate) async fn spawn_qbittorrent_web(
        &self,
    ) -> QBittorrentResult<QBittorrentClientProcess> {
        self.create_profile().await?;

        let result = Command::new("qbittorrent-nox")
            .args([format!("--profile={}", self.profile_dir.to_string_lossy())])
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn();

        let mut child = match result {
            Ok(child) => child,
            Err(err) => {
                let error_message = err.to_string();
                if error_message.contains("No such file or directory") {
                    return Err(QBittorrentError::QBittorrentNoxNotInstalled);
                }

                return Err(QBittorrentError::CantSpawnQBittorrent(error_message.into()));
            }
        };

        let stdout = child
            .stdout
            .take()
            .expect("QBittorrent doesn't have stdout");

        let mut stdout_lines = BufReader::new(stdout).lines();

        let process_handle = tokio::spawn(async move {
            child.wait().await.expect("QBittorrent client was killed");
        });

        while let Ok(line) = stdout_lines.next_line().await {
            let line = match line {
                Some(line) => line,
                None => continue,
            };

            if line.contains("To control qBittorrent, access the WebUI") {
                let port = Self::extract_port(&line).ok_or_else(|| {
                    QBittorrentError::CantSpawnQBittorrent(
                        format!("Can't extract port value from stdout: {}", line).into(),
                    )
                })?;

                info!("Spawned QBitorrent web API at http://localhost:{port}");

                return Ok(QBittorrentClientProcess {
                    process_handle,
                    port,
                });
            }
        }

        Err(QBittorrentError::QBittorrentDidntPrintReady)
    }

    async fn create_profile(&self) -> QBittorrentResult<()> {
        use QBittorrentError as Error;

        let config_dir = {
            let mut config_dir = self.profile_dir.clone();
            config_dir.push("qBittorrent");
            config_dir.push("config");
            config_dir
        };

        tokio::fs::create_dir_all(&config_dir)
            .await
            .map_err(|err| {
                Error::CantSpawnQBittorrent(
                    format!(
                        "Couldn't create qBittorrent profile config dir at {}. reason: {err}",
                        config_dir.to_str().unwrap()
                    )
                    .into(),
                )
            })?;

        let ini_file_path = {
            let mut ini_file_path = config_dir.clone();
            ini_file_path.push("qBittorrent.ini");
            ini_file_path
        };

        if tokio::fs::try_exists(&ini_file_path).await.map_err(|err| {
            Error::CantGenerateProfile(
                format!(
                    "Couldn't access the qBittorrent ini file path at {}. reason: {err}",
                    ini_file_path.to_str().unwrap()
                )
                .into(),
            )
        })? && tokio::fs::remove_file(&ini_file_path).await.is_err()
        {
            return Err(Error::CantSpawnQBittorrent(
                format!(
                    "Couldn't remove previous qBittorrent.ini file at {}",
                    ini_file_path.to_str().unwrap()
                )
                .into(),
            ));
        }

        let mut ini_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&ini_file_path)
            .await
            .map_err(|err| {
                Error::CantGenerateProfile(
                    format!(
                        "Couldn't create and open qBittorrent.ini file at {}. reason: {err}",
                        ini_file_path.to_str().unwrap()
                    )
                    .into(),
                )
            })?;

        ini_file
            .write_all(QBITTORRENT_INI_FILE_CONTENTS.as_bytes())
            .await
            .map_err(|err| {
                Error::CantGenerateProfile(
                    format!(
                        "Couldn't write to qBittorrent.ini file at {}. reason: {err}",
                        ini_file_path.to_str().unwrap()
                    )
                    .into(),
                )
            })?;

        Ok(())
    }

    fn extract_port(message: &str) -> Option<usize> {
        message
            .split(':')
            .next_back()
            .and_then(|last_element| last_element.parse().ok())
    }
}

#[derive(Debug)]
pub enum QBittorrentClientMessage {
    AddTorrent {
        hash: Box<str>,
        extra: Box<TorrentExtra>,
        result_sender: tokio::sync::oneshot::Sender<QBittorrentWebApiResult<()>>,
    },
    RemoveTorrent {
        id: Box<str>,
        result_sender: tokio::sync::oneshot::Sender<QBittorrentWebApiResult<()>>,
    },
    SetExtra {
        id: Box<str>,
        extra: Box<TorrentExtra>,
        result_sender: tokio::sync::oneshot::Sender<QBittorrentWebApiResult<()>>,
    },
    UpdateTorrentList {
        result_sender: tokio::sync::oneshot::Sender<QBittorrentWebApiResult<()>>,
    },
}

impl Drop for QBittorrentClientProcess {
    fn drop(&mut self) {
        self.process_handle.abort();
    }
}

pub type QBittorrentResult<T> = Result<T, QBittorrentError>;

#[derive(Debug)]
pub enum QBittorrentError {
    QBittorrentNoxNotInstalled,
    CantSpawnQBittorrent(Box<str>),
    QBittorrentDidntPrintReady,
    CantGenerateProfile(Box<str>),
}

impl Display for QBittorrentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            QBittorrentError::QBittorrentNoxNotInstalled => "qbittorrent-nox isn't installed",
            QBittorrentError::CantSpawnQBittorrent(err) => err,
            QBittorrentError::QBittorrentDidntPrintReady => {
                "qbittorrent-nox didn't print ready message"
            }
            QBittorrentError::CantGenerateProfile(err) => err,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::qbittorrent_client::QBittorrentClient;

    #[tokio::test]
    async fn test_spawn_process() {
        let client = QBittorrentClient::try_new(None).unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);
    }

    #[test]
    fn test_extract_port() {
        assert_eq!(QBittorrentClient::extract_port(""), None);
        assert_eq!(QBittorrentClient::extract_port(":1234"), Some(1234));
        assert_eq!(QBittorrentClient::extract_port(":hey"), None);
        assert_eq!(
            QBittorrentClient::extract_port(
                "To control qBittorrent, access the WebUI at http://127.0.0.1:8472"
            ),
            Some(8472)
        );
    }
}
