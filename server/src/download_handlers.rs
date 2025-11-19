use super::State;
use axum::{Json, extract, http::StatusCode};
use domain::{Download, DownloadForm, DownloadState, series::EditSeriesFileMappingForm};
use log::{debug, error, info};
use std::path::PathBuf;
use torrent::{TorrentExtra, qbittorrent_client::QBittorrentClientMessage};

pub async fn watch_and_process_downloads(
    media_dir: PathBuf,
    crate::AppState {
        media_signal_watcher,
        mut download_signal_watcher,
        processing_list_watcher,
    }: crate::AppState,
) {
    // TODO REFACTOR THIS AS A WHOLE
    loop {
        let hashes: Box<[_]> = {
            let (torrents_to_process, torrents_with_missing_files) = {
                let torrents = download_signal_watcher.data.borrow_and_update().clone();
                let processed_torrents = processing_list_watcher.data.borrow();

                // TODO REMOVE ME
                let torrents_to_process: Box<_> = torrents
                    .clone()
                    .into_iter()
                    .filter(|torrent| torrent.state.is_done())
                    .filter(|torrent| {
                        match torrent.try_into().ok() as Option<TorrentExtra> {
                            Some(extra) => !extra.needs_file_mapping(),
                            None => {
                                // TODO do proper error handling here
                                error!("Couldn't extract extra from torrent category");
                                false
                            }
                        }
                    })
                    .filter(|torrent| !processed_torrents.contains(&torrent.hash))
                    .collect();

                let updated_processed_torrents: Vec<Box<str>> = {
                    let mut vec =
                        Vec::with_capacity(processed_torrents.len() + torrents_to_process.len());
                    vec.extend_from_slice(&processed_torrents);
                    vec.extend(
                        torrents_to_process
                            .iter()
                            .map(|torrent| torrent.hash.clone()),
                    );

                    vec
                };

                // TODO LOG THESE FILES
                let torrents_with_missing_files: Box<[Box<str>]> = torrents
                    .into_iter()
                    .filter_map(|torrent| {
                        if matches!(torrent.state, torrent::TorrentState::MissingFiles) {
                            Some(torrent.hash.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                drop(processed_torrents);
                // TODO remove let _
                let _ = processing_list_watcher
                    .updater
                    .send(updated_processed_torrents.into());

                (torrents_to_process, torrents_with_missing_files)
            };

            let futures =
                torrents_to_process
                    .into_iter()
                    .map(async |torrent| -> Option<Box<str>> {
                        let extra: TorrentExtra = torrent
                            .as_ref()
                            .try_into()
                            .inspect_err(|err| {
                                error!("Couldn't extract extra from torrent's category. {err}")
                            })
                            .ok()?;
                        let title = extra.metadata_ref().title.clone();

                        info!("Preparing torrent named {}.", &torrent.name);

                        match extra {
                            TorrentExtra::Movie { ref metadata } => {
                                crate::prepare::prepare_movie(
                                    &media_dir,
                                    metadata,
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
                            }
                            TorrentExtra::Series {
                                ref metadata,
                                files_mapping_form,
                            } => {
                                crate::prepare::prepare_series(
                                    &media_dir,
                                    metadata,
                                    &torrent.save_path,
                                    files_mapping_form.expect("files mapping form was None."),
                                )
                                .await
                                .inspect_err(|err| {
                                    // TODO delete the files if this happens
                                    error!(
                                        "Couldn't prepare series with title {}. Reason: {err}.",
                                        title
                                    )
                                })
                                .ok()?;
                            }
                        }

                        Some(torrent.hash.clone())
                    });

            futures::future::join_all(futures)
                .await
                .into_iter()
                .flatten()
                .chain(torrents_with_missing_files.into_iter())
                .collect()
        };

        let removal_futures = hashes.iter().map(async |hash| {
            let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

            download_signal_watcher
                .signal_sender
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

        if download_signal_watcher.data.changed().await.is_err() {
            break;
        }

        if did_media_library_change {
            let _ = media_signal_watcher
                .signal_sender
                .send(())
                .await
                .inspect_err(|_| error!("Media library watcher loop was dropped."));
        }
    }

    unreachable!("Torrent channel was dropped")
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
        .download_signal_watcher
        .signal_sender
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
        .download_signal_watcher
        .signal_sender
        .send(QBittorrentClientMessage::UpdateTorrentList { result_sender })
        .await
        .unwrap();

    result_receiver.await.unwrap().unwrap();

    let processing_list = state.processing_list_watcher.data.borrow();

    Json(
        state
            .download_signal_watcher
            .data
            .borrow()
            .iter()
            .inspect(|torrents| debug!("Requested torrents list {:?}", torrents))
            .map(|torrent| torrent.clone().into())
            .map(|mut download: Download| {
                if processing_list.contains(&download.id) {
                    download.state = DownloadState::Processing;
                }
                download
            })
            .collect(),
    )
}

pub async fn add_download(
    extract::State(state): State,
    Json(form): Json<DownloadForm>,
) -> axum::response::Result<()> {
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

    state
        .download_signal_watcher
        .signal_sender
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
        .download_signal_watcher
        .signal_sender
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
    Json(file_mapping_form): Json<
        EditSeriesFileMappingForm<domain::series::file_mapping_form_state::NeedsValidation>,
    >,
) -> axum::response::Result<()> {
    let contents = {
        let (contents_result_sender, contents_result_receiver) = tokio::sync::oneshot::channel();

        state
            .download_signal_watcher
            .signal_sender
            .send(QBittorrentClientMessage::GetTorrentContents {
                id: file_mapping_form.id.clone(),
                result_sender: contents_result_sender,
            })
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        contents_result_receiver
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map_err(|_| StatusCode::NOT_FOUND)?
    };

    let allowed_files: Box<_> = contents
        .into_iter()
        .map(|content| content.name.to_string())
        .collect();

    let valid_form = file_mapping_form
        .validate(&allowed_files)
        .ok_or(StatusCode::BAD_REQUEST)?;

    let current_extra: TorrentExtra = {
        let torrent_list = state.download_signal_watcher.data.borrow();
        let torrent = torrent_list
            .iter()
            .find(|torrent| torrent.hash == valid_form.id)
            .ok_or(StatusCode::NOT_FOUND)?;

        torrent.try_into().map_err(|_| {
            error!(
                "Detected faulty category string on torrent with name {}",
                torrent.name
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    };

    let id = valid_form.id.clone();

    let new_extra = match current_extra {
        TorrentExtra::Movie { .. } => return Err(StatusCode::BAD_REQUEST.into()),
        TorrentExtra::Series { metadata, .. } => TorrentExtra::Series {
            metadata,
            files_mapping_form: Some(valid_form),
        },
    };

    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
    state
        .download_signal_watcher
        .signal_sender
        .send(QBittorrentClientMessage::SetExtra {
            id,
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
