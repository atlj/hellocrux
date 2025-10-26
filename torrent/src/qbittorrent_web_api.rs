use reqwest::{Client, Url};

use crate::api_types::TorrentInfo;

const BASE_URL: &str = "http://127.0.0.1";

async fn add_torrent(client: &Client, port: usize, hash: &str) -> QBittorrentWebApiResult<()> {
    let url: Url = {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_port(Some(port as u16))
            .expect("Invalid port was passed");
        url.set_path("api/v2/torrents/add");
        url
    };

    let result = client
        .post(url)
        .form(&AddTorrentForm { urls: hash })
        .send()
        .await
        .map_err(|err| QBittorrentWebApiError::CouldntCallApi(err.to_string().into()))?
        .text()
        .await
        .map_err(|err| QBittorrentWebApiError::CantGetTextContent(err.to_string().into()))?;

    if result != "Ok." {
        return Err(QBittorrentWebApiError::CantAddTorrent(
            format!("Api returned a non 'Ok.' body {result}").into(),
        ));
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct AddTorrentForm<'a> {
    urls: &'a str,
}

async fn get_torrent_list(
    client: &Client,
    port: usize,
) -> QBittorrentWebApiResult<Box<[TorrentInfo]>> {
    let url: Url = {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_port(Some(port as u16))
            .expect("Invalid port was passed");
        url.set_path("api/v2/torrents/info");
        url
    };

    let result = client
        .get(url)
        .send()
        .await
        .map_err(|err| QBittorrentWebApiError::CouldntCallApi(err.to_string().into()))?
        .text()
        .await
        .map_err(|err| QBittorrentWebApiError::CantGetTextContent(err.to_string().into()))?;

    serde_json::from_str(&result)
        .map_err(|err| QBittorrentWebApiError::CantDeserialize(err.to_string().into()))
}

pub type QBittorrentWebApiResult<T> = Result<T, QBittorrentWebApiError>;

#[derive(Debug)]
pub enum QBittorrentWebApiError {
    CouldntCallApi(Box<str>),
    CantGetTextContent(Box<str>),
    CantDeserialize(Box<str>),
    CantAddTorrent(Box<str>),
}

#[cfg(test)]
mod tests {
    use crate::{
        qbittorrent_client::QBittorrentClient,
        qbittorrent_web_api::{QBittorrentWebApiError, add_torrent, get_torrent_list},
    };

    #[tokio::test]
    async fn test_add_torrent() {
        let client = QBittorrentClient::try_new().unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);

        let http_client = reqwest::Client::new();

        add_torrent(&http_client, client_process.port, "https://cdimage.debian.org/debian-cd/current/arm64/bt-cd/debian-13.1.0-arm64-netinst.iso.torrent").await.unwrap();

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        assert!(!torrent_list.is_empty());

        dbg!(&torrent_list);
    }

    #[tokio::test]
    async fn test_add_faulty_torrent() {
        let client = QBittorrentClient::try_new().unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);

        let http_client = reqwest::Client::new();

        assert!(matches!(
            add_torrent(
                &http_client,
                client_process.port,
                "non_existent_link_for_torrent",
            )
            .await,
            Err(QBittorrentWebApiError::CantAddTorrent(_))
        ));
    }

    #[tokio::test]
    async fn test_get_torrent_list() {
        let client = QBittorrentClient::try_new().unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);

        let http_client = reqwest::Client::new();

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        dbg!(&torrent_list);
    }
}
