use std::{collections::HashMap, path::PathBuf};

use axum::{Json, extract, http::StatusCode, response::IntoResponse};
use domain::{
    series::EpisodeIdentifier,
    subtitles::{
        SearchSubtitlesQuery, SubtitleDownloadError, SubtitleDownloadForm, SubtitleRequest,
    },
};
use futures::FutureExt;
use subtitles::SubtitleProvider;

use crate::{
    State,
    service::subtitle::{SubtitleSignal, SubtitleSignalSender},
};

pub async fn search_subtitles(
    extract::State(state): State,
    extract::Query(query): extract::Query<SearchSubtitlesQuery>,
) -> axum::response::Result<axum::response::Json<Box<[subtitles::SubtitleDownloadOption<usize>]>>> {
    let episode_identifier = match (query.season_no, query.episode_no) {
        (Some(season_no), Some(episode_no)) => Some(EpisodeIdentifier {
            season_no,
            episode_no,
        }),
        _ => None,
    };

    let track_name: String = {
        let media_library = state.media_signal_watcher.data.borrow();
        let media = media_library
            .iter()
            .find(|media| media.id == query.media_id)
            .ok_or(StatusCode::NOT_FOUND)?;
        let media_paths = media
            .get_media_paths(episode_identifier.as_ref())
            .ok_or(StatusCode::NOT_FOUND)?;
        media_paths.track_name.clone()
    };

    let result = state
        .subtitle_provider
        .search(&track_name, query.language_code, episode_identifier)
        .await
        .map_err(|search_error| {
            axum::response::Response::builder()
                .status(500)
                .body(search_error.to_string())
                .expect("Response builder to not fail when there is no header")
        })?
        .collect();

    Ok(Json(result))
}

pub async fn download_subtitles(
    extract::State(state): State,
    axum::Json(SubtitleDownloadForm { media_id, requests }): axum::Json<SubtitleDownloadForm>,
) -> axum::response::Result<axum::Json<HashMap<usize, Result<(), SubtitleDownloadError>>>> {
    if requests.is_empty() {
        return Err(StatusCode::BAD_REQUEST.into());
    }

    // 1. Get all the paths for requests
    let request_path_pairs = {
        let media_library = state.media_signal_watcher.data.borrow();
        let media = media_library
            .iter()
            .find(|media| media.id == media_id)
            .ok_or(StatusCode::NOT_FOUND.into_response())?;

        requests
            .into_iter()
            .map(|request| {
                media
                    .get_media_paths(request.episode_identifier.as_ref())
                    .map(|media_paths| (request, state.media_dir.join(&media_paths.media)))
            })
            .collect::<Option<Vec<(SubtitleRequest, PathBuf)>>>()
    }
    .ok_or(StatusCode::BAD_REQUEST)?;

    // 2. Download
    let futures = request_path_pairs.into_iter().map(|(request, media_path)| {
        let subtitle_id = request.subtitle_id;
        download_subtitle(media_path, request, state.subtitle_signal_sender.clone())
            .map(move |future_result| future_result.map(|result| (subtitle_id, result)))
    });

    // 3. Check if any internal errors happened
    let result: axum::response::Result<Box<[(usize, Result<(), SubtitleDownloadError>)]>> =
        futures::future::join_all(futures)
            .await
            .into_iter()
            .collect();

    let download_results = result?;
    let len = download_results.len();

    // 4. Now update the media library
    state
        .media_signal_watcher
        .signal_sender
        .send(crate::service::media::MediaSignal::CrawlPartial { media_id })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(download_results.into_iter().fold(
        HashMap::with_capacity(len),
        |mut map, (id, result)| {
            map.insert(id, result);
            map
        },
    )))
}

async fn download_subtitle(
    media_path: PathBuf,
    request: SubtitleRequest,
    signal_sender: SubtitleSignalSender,
) -> axum::response::Result<Result<(), SubtitleDownloadError>> {
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

    let signal = SubtitleSignal::Download {
        media_path,
        result_sender,
        request,
    };

    signal_sender
        .send(signal)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = result_receiver
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(result)
}
