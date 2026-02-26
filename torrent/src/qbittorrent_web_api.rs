use std::fmt::Display;

use reqwest::{Client, StatusCode, Url};

use crate::{
    TorrentExtra,
    api_types::{TorrentContents, TorrentInfo},
};

const BASE_URL: &str = "http://127.0.0.1";

pub(crate) async fn add_torrent(
    client: &Client,
    port: usize,
    hash: &str,
    extra: &TorrentExtra,
) -> QBittorrentWebApiResult<()> {
    let url: Url = {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_port(Some(port as u16))
            .expect("Invalid port was passed");
        url.set_path("api/v2/torrents/add");
        url
    };

    let result = client
        .post(url)
        .form(&AddTorrentForm {
            urls: hash,
            category: encode_extra(extra)?.as_ref(),
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

pub(crate) async fn get_torrent_contents(
    client: &Client,
    port: usize,
    id: &str,
) -> QBittorrentWebApiResult<Box<[TorrentContents]>> {
    let url: Url = {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_port(Some(port as u16))
            .expect("Invalid port was passed");
        url.set_path("api/v2/torrents/files");
        url
    };

    let result = client
        .get(url)
        .query(&[("hash", id)])
        .send()
        .await
        .map_err(|err| match err.status() {
            Some(status_code) => QBittorrentWebApiError::NonOkStatus(status_code, format!("Non 200 status code returned from QBittorrent while trying to fetch torrent contents for id {id}").into()),
            _ => QBittorrentWebApiError::CouldntCallApi(err.to_string().into()),
        })?
        .text()
        .await
        .map_err(|err| QBittorrentWebApiError::CantGetTextContent(err.to_string().into()))?;

    serde_json::from_str(&result)
        .map_err(|err| QBittorrentWebApiError::CantDeserialize(err.to_string().into()))
}

pub(crate) async fn set_torrent_category(
    client: &Client,
    port: usize,
    id: &str,
    extra: &TorrentExtra,
) -> QBittorrentWebApiResult<()> {
    let category = &encode_extra(extra)?;
    add_torrent_category(client, port, category).await?;

    let url: Url = {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_port(Some(port as u16))
            .expect("Invalid port was passed");
        url.set_path("api/v2/torrents/setCategory");
        url
    };

    client
        .post(url)
        .form(&SetCategoryForm {
            category,
            hashes: id,
        })
        .send()
        .await
        .map_err(|err| QBittorrentWebApiError::CouldntCallApi(err.to_string().into()))?
        .text()
        .await
        .map_err(|err| QBittorrentWebApiError::CantGetTextContent(err.to_string().into()))?;

    // TODO API returns "" as text if operation is successful. Return non-Ok val if it's
    // something else
    Ok(())
}

async fn add_torrent_category(
    client: &Client,
    port: usize,
    new_category: &str,
) -> QBittorrentWebApiResult<()> {
    let url: Url = {
        let mut url: Url = BASE_URL.parse().unwrap();
        url.set_port(Some(port as u16))
            .expect("Invalid port was passed");
        url.set_path("api/v2/torrents/createCategory");
        url
    };

    client
        .post(url)
        .form(&CreateCategoryForm {
            category: new_category,
        })
        .send()
        .await
        .map_err(|err| QBittorrentWebApiError::CouldntCallApi(err.to_string().into()))?
        .text()
        .await
        .inspect(|text| {
            dbg!(text);
        })
        .map_err(|err| QBittorrentWebApiError::CantGetTextContent(err.to_string().into()))?;

    Ok(())
}

fn encode_extra(extra: &TorrentExtra) -> QBittorrentWebApiResult<String> {
    let json_string = serde_json::to_string(extra).map_err(|err| {
        QBittorrentWebApiError::CantAddTorrent(
            format!("Can't serialize metadata {:?}. Reason: {err}", &extra).into(),
        )
    })?;

    Ok(domain::encode_decode::encode_url_safe(&json_string))
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

#[derive(serde::Serialize)]
struct CreateCategoryForm<'a> {
    category: &'a str,
}

#[derive(serde::Serialize)]
struct SetCategoryForm<'a> {
    hashes: &'a str,
    category: &'a str,
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
    NonOkStatus(StatusCode, Box<str>),
    CouldntCallApi(Box<str>),
    CantGetTextContent(Box<str>),
    CantDeserialize(Box<str>),
    CantAddTorrent(Box<str>),
    CantDeleteTorrent(Box<str>),
}

impl Display for QBittorrentWebApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            QBittorrentWebApiError::NonOkStatus(_, msg) => msg,
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
    use std::{collections::HashMap, marker::PhantomData, time::Duration};

    use domain::{MediaMetaData, series::EditSeriesFileMappingForm};

    use crate::{
        TorrentExtra,
        qbittorrent_client::QBittorrentClient,
        qbittorrent_web_api::{
            QBittorrentWebApiError, add_torrent, get_torrent_contents, get_torrent_list,
            remove_torrent, set_torrent_category,
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
        add_torrent(&http_client, client_process.port, "https://cdimage.debian.org/debian-cd/current/arm64/bt-cd/debian-13.1.0-arm64-netinst.iso.torrent", &TorrentExtra::new(metadata, false)).await.unwrap();

        tokio::time::sleep(Duration::from_secs(5)).await;

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        assert!(!torrent_list.is_empty());

        dbg!(&torrent_list);
    }

    #[tokio::test]
    async fn test_set_torrent_category() {
        let client = QBittorrentClient::try_new(None).unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);

        let http_client = reqwest::Client::new();

        remove_torrent(&http_client, client_process.port, "all")
            .await
            .unwrap();

        let metadata = MediaMetaData {
            title: "My Movie".to_string(),
            thumbnail: "https://image.com".to_string(),
        };
        add_torrent(&http_client, client_process.port, "https://cdimage.debian.org/debian-cd/current/arm64/bt-cd/debian-13.1.0-arm64-netinst.iso.torrent", &TorrentExtra::new(metadata.clone(), false)).await.unwrap();

        tokio::time::sleep(Duration::from_secs(5)).await;

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        assert!(torrent_list.len() == 1);

        dbg!(&torrent_list);

        let first_id = &torrent_list[0].hash;

        set_torrent_category(
            &http_client,
            client_process.port,
            first_id,
            &TorrentExtra::Series {
                metadata,
                files_mapping_form: Some(EditSeriesFileMappingForm {
                    id: "hey".into(),
                    phantom: PhantomData,
                    file_mapping: HashMap::new(),
                }),
            },
        )
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_secs(5)).await;

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        assert!(torrent_list.len() == 1);

        assert_eq!(torrent_list[0].category, "Hello World".into());
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
                &TorrentExtra::new(metadata, false)
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
        add_torrent(&http_client, client_process.port, "https://cdimage.debian.org/debian-cd/current/arm64/bt-cd/debian-13.1.0-arm64-netinst.iso.torrent", &TorrentExtra::new(metadata, false)).await.unwrap();

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

    #[tokio::test]
    async fn test_get_torrent_contents() {
        let client = QBittorrentClient::try_new(None).unwrap();
        let client_process = client.spawn_qbittorrent_web().await.unwrap();

        dbg!(&client_process);

        let http_client = reqwest::Client::new();

        remove_torrent(&http_client, client_process.port, "all")
            .await
            .unwrap();

        let metadata = MediaMetaData {
            title: "My Movie".to_string(),
            thumbnail: "https://image.com".to_string(),
        };
        add_torrent(&http_client, client_process.port, "https://cdimage.debian.org/debian-cd/current/arm64/bt-cd/debian-13.1.0-arm64-netinst.iso.torrent", &TorrentExtra::new(metadata.clone(), false)).await.unwrap();

        tokio::time::sleep(Duration::from_secs(5)).await;

        let torrent_list = get_torrent_list(&http_client, client_process.port)
            .await
            .unwrap();

        assert!(!torrent_list.is_empty());

        dbg!(&torrent_list);

        let contents =
            get_torrent_contents(&http_client, client_process.port, &torrent_list[0].hash)
                .await
                .unwrap();

        assert!(!contents.is_empty());

        dbg!(&contents);
    }
}
