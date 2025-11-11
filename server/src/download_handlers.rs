use super::State;
use axum::{Json, extract, http::StatusCode};
use domain::{Download, DownloadForm, MediaMetaData};
use log::{debug, error, info};
use std::{collections::HashSet, path::PathBuf};
use tokio::task::JoinHandle;
use torrent::{
    TorrentInfo,
    qbittorrent_client::{QBittorrentClient, QBittorrentClientMessage},
};

pub async fn watch_and_process_downloads(
    media_dir: PathBuf,
    mut receiver: tokio::sync::watch::Receiver<Box<[TorrentInfo]>>,
    sender: tokio::sync::mpsc::Sender<QBittorrentClientMessage>,
    media_update_request_sender: tokio::sync::mpsc::Sender<()>,
) {
    let mut processed_hashes: HashSet<Box<str>> = HashSet::new();

    loop {
        let hashes: Box<[_]> = {
            let torrents = receiver.borrow_and_update().clone();

            let futures = torrents
                .into_iter()
                .filter(|torrent| torrent.state.is_done())
                .filter(|torrent| !processed_hashes.contains(&torrent.hash))
                .map(async |torrent| -> Option<Box<str>> {
                    let metadata: MediaMetaData = torrent
                        .as_ref()
                        .try_into()
                        .inspect_err(|err| {
                            error!("Couldn't extract metadata from torrent's category. {err}")
                        })
                        .ok()?;
                    info!("Preparing torrent named {}.", &torrent.name);

                    crate::prepare::prepare_movie(&media_dir, &metadata, &torrent.content_path)
                        .await
                        .inspect_err(|err| {
                            error!(
                                "Couldn't prepare movie with title {}. Reason: {err}.",
                                &metadata.title
                            )
                        })
                        .ok()?;

                    Some(torrent.hash.clone())
                });

            futures::future::join_all(futures)
                .await
                .into_iter()
                .flatten()
                .collect()
        };

        let removal_futures = hashes.iter().map(async |hash| {
            let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

            sender
                .send(QBittorrentClientMessage::RemoveTorrent {
                    id: hash.clone(),
                    result_sender,
                })
                .await
                .inspect_err(|err| error!("QBittorrent Client was dropped. Reason: {err}"))
                .ok()?;

            result_receiver
                .await
                .inspect_err(|err| error!("QBittorrent Client was dropped. Reason: {err}"))
                .ok()?
                .inspect_err(|err| {
                    error!("Couldn't remove torrent with hash {hash}. Reason: {err}")
                })
                .ok()?;

            Some(())
        });

        futures::future::join_all(removal_futures).await;

        let did_media_library_change = !hashes.is_empty();
        // TODO delete missing torrents
        processed_hashes.extend(hashes);

        if receiver.changed().await.is_err() {
            break;
        }

        if did_media_library_change {
            let _ = media_update_request_sender
                .send(())
                .await
                .inspect_err(|_| error!("Media library watcher loop was dropped."));
        }
    }

    unreachable!("Torrent channel was dropped")
}

pub async fn spawn_download_event_loop(
    path: PathBuf,
) -> (
    tokio::sync::mpsc::Sender<QBittorrentClientMessage>,
    tokio::sync::watch::Receiver<Box<[TorrentInfo]>>,
    JoinHandle<()>,
) {
    let client = QBittorrentClient::try_new(Some(path)).unwrap();
    let (sender, receiver) = tokio::sync::mpsc::channel(100);
    let (list_sender, list_receiver) =
        tokio::sync::watch::channel::<Box<[TorrentInfo]>>(Box::new([]));

    let handle = tokio::spawn(async move {
        client
            .event_loop(receiver, list_sender)
            .await
            .expect("Event loop exited sooner than expected");
    });

    // TODO remove unwraps
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
    sender
        .send(QBittorrentClientMessage::UpdateTorrentList { result_sender })
        .await
        .unwrap();

    result_receiver.await.unwrap().unwrap();

    (sender, list_receiver, handle)
}

pub async fn get_downloads(extract::State(state): State) -> Json<Box<[Download]>> {
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

    // TODO: make this a periodic call.
    state
        .download_channels
        .0
        .send(QBittorrentClientMessage::UpdateTorrentList { result_sender })
        .await
        .unwrap();

    result_receiver.await.unwrap().unwrap();

    Json(
        state
            .download_channels
            .1
            .borrow()
            .iter()
            .inspect(|torrents| debug!("Requested torrents list {:?}", torrents))
            .map(|torrent| torrent.clone().into())
            .collect(),
    )
}

pub async fn add_download(
    extract::State(state): State,
    Json(form): Json<DownloadForm>,
) -> axum::response::Result<()> {
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

    state
        .download_channels
        .0
        .send(QBittorrentClientMessage::AddTorrent {
            hash: form.hash,
            result_sender,
            metadata: Box::new(form.metadata),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = result_receiver
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

pub async fn remove_download(
    extract::State(state): State,
    body: String,
) -> axum::response::Result<()> {
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

    state
        .download_channels
        .0
        .send(QBittorrentClientMessage::RemoveTorrent {
            id: body.into(),
            result_sender,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = result_receiver
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

pub async fn pause_download() -> axum::response::Result<()> {
    todo!()
}

pub async fn resume_download() -> axum::response::Result<()> {
    todo!()
}
