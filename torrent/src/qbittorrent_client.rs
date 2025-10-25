use std::env;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct QBittorrentClient {
    process_handle: Option<JoinHandle<()>>,
}

impl QBittorrentClient {
    pub fn try_new() -> QBittorrentResult<Self> {
        Ok(Self {
            process_handle: None,
        })
    }

    async fn spawn_qbittorrent_web(&mut self) -> QBittorrentResult<()> {
        let profile_path = {
            let mut current_dir = env::current_dir().expect("Current working dir is invalid");
            current_dir.push("qbittorrent_config");
            current_dir
        };

        let result = Command::new("qbittorrent-nox")
            .args([format!("--profile={}", profile_path.to_string_lossy())])
            .stdout(Stdio::piped())
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

        self.process_handle = Some(tokio::spawn(async move {
            child.wait().await.expect("QBittorrent client was killed");
        }));

        while let Ok(line) = stdout_lines.next_line().await {
            let line = match line {
                Some(line) => line,
                None => continue,
            };

            if line.contains("To control qBittorrent, access the WebUI") {
                return Ok(());
            }
        }

        Err(QBittorrentError::QBittorrentDidntPrintReady)
    }
}

impl Drop for QBittorrentClient {
    fn drop(&mut self) {
        self.process_handle.take().map(|handle| handle.abort());
    }
}

pub type QBittorrentResult<T> = Result<T, QBittorrentError>;

#[derive(Debug)]
pub enum QBittorrentError {
    QBittorrentNoxNotInstalled,
    CantSpawnQBittorrent(Box<str>),
    QBittorrentDidntPrintReady,
}

#[cfg(test)]
mod tests {
    use crate::qbittorrent_client::QBittorrentClient;

    #[tokio::test]
    async fn test_spawn_child() {
        let mut client = QBittorrentClient::try_new().unwrap();
        client.spawn_qbittorrent_web().await.unwrap();
    }
}
