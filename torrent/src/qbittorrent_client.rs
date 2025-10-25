use std::env;
use std::path::PathBuf;
use std::process::Stdio;

use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct QBittorrentClient {
    process_handle: Option<JoinHandle<()>>,
    profile_dir: PathBuf,
}

const QBITTORRENT_INI_FILE_CONTENTS: &str = r#"[LegalNotice]
Accepted=true

[Preferences]
General\Locale=en
MailNotification\req_auth=true
WebUI\Address=127.0.0.1
WebUI\LocalHostAuth=false
WebUI\Port=45432
"#;

impl QBittorrentClient {
    pub fn try_new() -> QBittorrentResult<Self> {
        Ok(Self {
            process_handle: None,
            profile_dir: env::temp_dir().join("streamy-qbittorrent"),
        })
    }

    async fn spawn_qbittorrent_web(&mut self) -> QBittorrentResult<()> {
        self.create_profile().await;

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

    async fn create_profile(&self) {
        let config_dir = {
            let mut config_dir = self.profile_dir.clone();
            config_dir.push("qBittorrent");
            config_dir.push("config");
            config_dir
        };

        tokio::fs::create_dir_all(&config_dir)
            .await
            .unwrap_or_else(|_| panic!("Couldn't create qBittorrent profile config dir at {}",
                config_dir.to_str().unwrap()));

        let ini_file_path = {
            let mut ini_file_path = config_dir.clone();
            ini_file_path.push("qBittorrent.ini");
            ini_file_path
        };

        if tokio::fs::try_exists(&ini_file_path).await.unwrap_or_else(|_| panic!("Couldn't access the qBittorrent ini file path at {}",
            ini_file_path.to_str().unwrap())) && tokio::fs::remove_file(&ini_file_path).await.is_err() {
            panic!(
                "Couldn't remove previous qBittorrent.ini file at {}",
                ini_file_path.to_str().unwrap()
            )
        }

        let mut ini_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&ini_file_path)
            .await
            .unwrap_or_else(|_| panic!("Couldn't create and open qBittorrent.ini file at {}",
                ini_file_path.to_str().unwrap()));

        ini_file
            .write_all(QBITTORRENT_INI_FILE_CONTENTS.as_bytes())
            .await
            .unwrap_or_else(|_| panic!("Couldn't write to qBittorrent.ini file at {}",
                ini_file_path.to_str().unwrap()));
    }
}

impl Drop for QBittorrentClient {
    fn drop(&mut self) {
        if let Some(handle) = self.process_handle.take() { handle.abort() }
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
