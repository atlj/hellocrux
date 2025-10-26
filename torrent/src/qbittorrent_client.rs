use std::env;
use std::path::PathBuf;
use std::process::Stdio;

use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;

use crate::api_types::TorrentInfo;
use crate::qbittorrent_web_api::{QBittorrentWebApiResult, add_torrent, get_torrent_list};

#[derive(Debug)]
pub struct QBittorrentClient {
    pub profile_dir: PathBuf,
    client_process: Option<QBittorrentClientProcess>,
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
            client_process: None,
            profile_dir: profile_dir.unwrap_or(env::temp_dir().join("streamy-qbittorrent")),
        })
    }

    pub async fn event_loop(
        &self,
        mut receiver: tokio::sync::mpsc::Receiver<QBittorrentClientMessage<'_>>,
        state_updater: tokio::sync::watch::Sender<Box<[TorrentInfo]>>,
    ) -> QBittorrentResult<()> {
        let process_client = self.spawn_qbittorrent_web().await?;
        let http_client = reqwest::Client::new();

        loop {
            let message = if let Some(message) = receiver.recv().await {
                message
            } else {
                break;
            };

            match message {
                QBittorrentClientMessage::AddTorrent {
                    hash,
                    result_sender,
                } => {
                    let result = add_torrent(&http_client, process_client.port, hash).await;
                    // TODO: add logging here
                    let _ = result_sender.send(result);
                }
                QBittorrentClientMessage::UpdateTorrentList { result_sender } => {
                    match get_torrent_list(&http_client, process_client.port).await {
                        Ok(torrent_list) => {
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
pub enum QBittorrentClientMessage<'a> {
    AddTorrent {
        hash: &'a str,
        result_sender: tokio::sync::oneshot::Sender<QBittorrentWebApiResult<()>>,
    },
    UpdateTorrentList {
        result_sender: tokio::sync::oneshot::Sender<QBittorrentWebApiResult<()>>,
    },
}

impl Drop for QBittorrentClient {
    fn drop(&mut self) {
        if let Some(process) = self.client_process.take() {
            process.process_handle.abort();
        }
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

#[cfg(test)]
mod tests {
    use crate::qbittorrent_client::QBittorrentClient;

    #[tokio::test]
    async fn test_spawn_process() {
        let mut client = QBittorrentClient::try_new(None).unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();
        client.client_process = Some(client_process);

        dbg!(&client.client_process);
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
