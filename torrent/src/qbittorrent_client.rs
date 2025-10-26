use std::env;
use std::path::PathBuf;
use std::process::Stdio;

use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct QBittorrentClient {
    profile_dir: PathBuf,
    client_process: Option<QBittorrentClientProcess>,
}

#[derive(Debug)]
struct QBittorrentClientProcess {
    process_handle: JoinHandle<()>,
    port: usize,
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
    pub fn try_new() -> QBittorrentResult<Self> {
        Ok(Self {
            client_process: None,
            profile_dir: env::temp_dir().join("streamy-qbittorrent"),
        })
    }

    async fn spawn_qbittorrent_web(&self) -> QBittorrentResult<QBittorrentClientProcess> {
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

        tokio::fs::create_dir_all(&config_dir).await.map_err(|_| {
            Error::CantSpawnQBittorrent(
                format!(
                    "Couldn't create qBittorrent profile config dir at {}",
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

        if tokio::fs::try_exists(&ini_file_path).await.map_err(|_| {
            Error::CantGenerateProfile(
                format!(
                    "Couldn't access the qBittorrent ini file path at {}",
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
            .map_err(|_| {
                Error::CantGenerateProfile(
                    format!(
                        "Couldn't create and open qBittorrent.ini file at {}",
                        ini_file_path.to_str().unwrap()
                    )
                    .into(),
                )
            })?;

        ini_file
            .write_all(QBITTORRENT_INI_FILE_CONTENTS.as_bytes())
            .await
            .map_err(|_| {
                Error::CantGenerateProfile(
                    format!(
                        "Couldn't write to qBittorrent.ini file at {}",
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
    async fn test_spawn_child() {
        let mut client = QBittorrentClient::try_new().unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();
        client.client_process = Some(client_process);

        dbg!(&client.client_process);
        if client.client_process.is_none() {
            panic!("client_process wasn't set")
        }
    }

    #[tokio::test]
    async fn test_get_torrents() {
        let mut client = QBittorrentClient::try_new().unwrap();
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
