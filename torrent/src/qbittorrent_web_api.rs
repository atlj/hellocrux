use std::fmt::Display;

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use domain::MediaMetaData;
use reqwest::{Client, Url};

use crate::api_types::TorrentInfo;

const BASE_URL: &str = "http://127.0.0.1";

pub(crate) async fn add_torrent(
    client: &Client,
    port: usize,
    hash: &str,
    metadata: &MediaMetaData,
) -> QBittorrentWebApiResult<()> {
    let url: Url = {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_port(Some(port as u16))
            .expect("Invalid port was passed");
        url.set_path("api/v2/torrents/add");
        url
    };

    let category_string = {
        let json_string = serde_json::to_string(metadata).map_err(|err| {
            QBittorrentWebApiError::CantAddTorrent(
                format!("Can't serialize metadata {:?}. Reason: {err}", &metadata).into(),
            )
        })?;

        URL_SAFE.encode(json_string)
    };

    let result = client
        .post(url)
        .form(&AddTorrentForm {
            urls: hash,
            category: &category_string,
            root_folder: true,
        })
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

pub(crate) async fn remove_torrent(
    client: &Client,
    port: usize,
    id: &str,
) -> QBittorrentWebApiResult<()> {
    let url: Url = {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_port(Some(port as u16))
            .expect("Invalid port was passed");
        url.set_path("api/v2/torrents/delete");
        url
    };

    client
        .post(url)
        .form(&RemoveTorrentForm {
            hashes: id,
            delete_files: true,
        })
        .send()
        .await
        .map_err(|err| QBittorrentWebApiError::CouldntCallApi(err.to_string().into()))?
        .text()
        .await
        .map_err(|err| QBittorrentWebApiError::CantGetTextContent(err.to_string().into()))?;

    Ok(())
}

#[derive(serde::Serialize)]
struct AddTorrentForm<'a> {
    urls: &'a str,
    category: &'a str,
    root_folder: bool,
}

#[derive(serde::Serialize)]
struct RemoveTorrentForm<'a> {
    hashes: &'a str,
    #[serde(rename = "deleteFiles")]
    delete_files: bool,
}

pub(crate) async fn get_torrent_list(
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
    CantDeleteTorrent(Box<str>),
}

impl Display for QBittorrentWebApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            QBittorrentWebApiError::CouldntCallApi(msg) => msg,
            QBittorrentWebApiError::CantGetTextContent(msg) => msg,
            QBittorrentWebApiError::CantDeserialize(msg) => msg,
            QBittorrentWebApiError::CantAddTorrent(msg) => msg,
            QBittorrentWebApiError::CantDeleteTorrent(msg) => msg,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use domain::MediaMetaData;

    use crate::{
        qbittorrent_client::QBittorrentClient,
        qbittorrent_web_api::{
            QBittorrentWebApiError, add_torrent, get_torrent_list, remove_torrent,
        },
    };

    #[tokio::test]
    async fn test_add_torrent() {
        let client = QBittorrentClient::try_new(None).unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);

        let http_client = reqwest::Client::new();

        let metadata = MediaMetaData {
            title: "My Movie".to_string(),
            thumbnail: "https://image.com".to_string(),
        };
        add_torrent(&http_client, client_process.port, "https://cdimage.debian.org/debian-cd/current/arm64/bt-cd/debian-13.1.0-arm64-netinst.iso.torrent", &metadata).await.unwrap();

        tokio::time::sleep(Duration::from_secs(5)).await;

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        assert!(!torrent_list.is_empty());

        dbg!(&torrent_list);
    }

    #[tokio::test]
    async fn test_add_faulty_torrent() {
        let client = QBittorrentClient::try_new(None).unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);

        let http_client = reqwest::Client::new();

        let metadata = MediaMetaData {
            title: "My Movie".to_string(),
            thumbnail: "https://image.com".to_string(),
        };
        assert!(matches!(
            add_torrent(
                &http_client,
                client_process.port,
                "non_existent_link_for_torrent",
                &metadata
            )
            .await,
            Err(QBittorrentWebApiError::CantAddTorrent(_))
        ));
    }

    #[tokio::test]
    async fn test_remove_torrent() {
        let client = QBittorrentClient::try_new(None).unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);

        let http_client = reqwest::Client::new();

        let metadata = MediaMetaData {
            title: "My Movie".to_string(),
            thumbnail: "https://image.com".to_string(),
        };
        add_torrent(&http_client, client_process.port, "https://cdimage.debian.org/debian-cd/current/arm64/bt-cd/debian-13.1.0-arm64-netinst.iso.torrent", &metadata).await.unwrap();

        tokio::time::sleep(Duration::from_secs(5)).await;

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        assert!(!torrent_list.is_empty());

        dbg!(&torrent_list);

        let first_item_hash = &torrent_list.first().unwrap().hash;

        remove_torrent(&http_client, client_process.port, first_item_hash)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(5)).await;

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        assert!(
            !torrent_list
                .iter()
                .any(|torrent| torrent.hash == *first_item_hash)
        );

        dbg!(&torrent_list);
    }

    #[tokio::test]
    async fn test_get_torrent_list() {
        let client = QBittorrentClient::try_new(None).unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);

        let http_client = reqwest::Client::new();

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        dbg!(&torrent_list);
    }
}
