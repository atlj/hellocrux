use super::State;
use axum::{Json, extract, http::StatusCode};
use domain::{Download, DownloadForm, EditSeriesFileMappingForm};
use log::{debug, error, info};
use std::{collections::HashSet, path::PathBuf};
use tokio::task::JoinHandle;
use torrent::{
    TorrentExtra, TorrentInfo,
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
                    let extra: TorrentExtra = torrent
                        .as_ref()
                        .try_into()
                        .inspect_err(|err| {
                            error!("Couldn't extract extra from torrent's category. {err}")
                        })
                        .ok()?;

                    if extra.needs_file_mapping() {
                        return None;
                    }

                    info!("Preparing torrent named {}.", &torrent.name);

                    crate::prepare::prepare_movie(
                        &media_dir,
                        extra.metadata_ref(),
                        &torrent.content_path,
                    )
                    .await
                    .inspect_err(|err| {
                        error!(
                            "Couldn't prepare movie with title {}. Reason: {err}.",
                            extra.metadata_ref().title
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TorrentContentsQuery {
    id: Box<str>,
}

pub async fn get_torrent_contents(
    extract::State(state): State,
    extract::Query(query): extract::Query<TorrentContentsQuery>,
) -> axum::response::Result<Json<Box<[Box<str>]>>> {
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

    // TODO: make this a periodic call.
    state
        .download_channels
        .0
        .send(QBittorrentClientMessage::GetTorrentContents {
            id: query.id.clone(),
            result_sender,
        })
        .await
        .unwrap();

    let contents = result_receiver.await.unwrap().map_err(|err| match err {
        // TODO we're not returning 404.
        torrent::QBittorrentWebApiError::NonOkStatus(status_code, ..) => status_code,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    debug!(
        "Requested contents list of torrent with id {} {:?}",
        &query.id, &contents
    );

    Ok(Json(
        contents.into_iter().map(|contents| contents.name).collect(),
    ))
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
            extra: Box::new(TorrentExtra::new(form.metadata, form.is_series)),
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

pub async fn update_file_mapping(
    extract::State(state): State,
    Json(file_mapping_form): Json<EditSeriesFileMappingForm>,
) -> axum::response::Result<()> {
    let current_extra: TorrentExtra = {
        let torrent_list = state.download_channels.1.borrow();
        let torrent = torrent_list
            .iter()
            .find(|torrent| torrent.hash == file_mapping_form.id)
            .ok_or(StatusCode::NOT_FOUND)?;

        torrent.try_into().map_err(|_| {
            error!(
                "Detected faulty category string on torrent with name {}",
                torrent.name
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    };

    let new_extra = match current_extra {
        TorrentExtra::Movie { .. } => return Err(StatusCode::BAD_REQUEST.into()),
        TorrentExtra::Series { metadata, .. } => TorrentExtra::Series {
            metadata,
            files_mapping: Some(file_mapping_form.file_mapping),
        },
    };

    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
    state
        .download_channels
        .0
        .send(QBittorrentClientMessage::SetExtra {
            id: file_mapping_form.id.clone(),
            extra: Box::new(new_extra),
            result_sender,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result_receiver
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

pub async fn pause_download() -> axum::response::Result<()> {
    todo!()
}

pub async fn resume_download() -> axum::response::Result<()> {
    todo!()
}
