use super::State;
use axum::{Json, extract, http::StatusCode};
use domain::{Download, DownloadForm, DownloadState, series::EditSeriesFileMappingForm};
use log::{debug, error};
use torrent::{TorrentExtra, qbittorrent_client::QBittorrentClientMessage};

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
