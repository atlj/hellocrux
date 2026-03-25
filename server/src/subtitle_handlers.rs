use std::{collections::HashMap, path::PathBuf};

use axum::{Json, extract, http::StatusCode, response::IntoResponse};
use domain::subtitles::{SubtitleProvider, SubtitleSearchResponse};
use domain::{
    language::LanguageCode,
    series::EpisodeIdentifier,
    subtitles::{
        SubtitleDownloadError, SubtitleDownloadForm, SubtitleDownloadResponse, SubtitleSearchForm,
        SubtitleSelection,
    },
};
use futures::FutureExt;

use crate::{
    State,
    service::subtitle::{SubtitleSignal, SubtitleSignalSender},
};

pub async fn search_subtitles(
    extract::State(state): State,
    axum::Json(form): axum::Json<SubtitleSearchForm>,
) -> axum::response::Result<axum::Json<SubtitleSearchResponse>> {
    // 1. Use the given form to create params for search API
    // Scoped so we drop  the media library handle as soon as possible
    let search_params = {
        let media_library = state.media_signal_watcher.data.borrow();
        let media = media_library
            .iter()
            .find(|media| media.id == form.media_id)
            .ok_or(StatusCode::NOT_FOUND)?;

        form.episode_identifiers
            .map(|identifiers| {
                identifiers
                    .into_iter()
                    .map(|identifier| {
                        let track_name: String = {
                            let media_paths = media
                                .get_media_paths(Some(&identifier))
                                .ok_or(StatusCode::NOT_FOUND.into_response())?;
                            media_paths.track_name.clone()
                        };
                        Ok((Some(identifier), track_name))
                    })
                    .collect::<axum::response::Result<Vec<(Option<EpisodeIdentifier>, String)>>>()
            })
            .unwrap_or_else(|| {
                let track_name: String = {
                    let media_paths = media
                        .get_media_paths(None)
                        .ok_or(StatusCode::NOT_FOUND.into_response())?;
                    media_paths.track_name.clone()
                };
                Ok(vec![(None, track_name)])
            })?
    };

    // 2. Convert the search params to search futures
    let search_futures = search_params
        .iter()
        .map(|(episode_identifier, track_name)| async {
            state
                .subtitle_provider
                .search(
                    track_name,
                    form.language_code.clone(),
                    episode_identifier.clone(),
                )
                .await
                .map(|result| result.collect::<Vec<_>>())
        });

    // 3. Drive search futures to completion
    let results = futures::future::join_all(search_futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|search_error| {
            axum::response::Response::builder()
                .status(500)
                .body(search_error.to_string())
                .expect("Response builder to not fail when there is no header")
        })?;

    Ok(Json(results))
}

pub async fn download_subtitles(
    extract::State(state): State,
    axum::Json(SubtitleDownloadForm {
        media_id,
        selections,
        language_code,
    }): axum::Json<SubtitleDownloadForm>,
) -> axum::response::Result<axum::Json<SubtitleDownloadResponse>> {
    if selections.is_empty() {
        return Err(StatusCode::BAD_REQUEST.into());
    }

    // 1. Get all the paths for requests
    let request_path_pairs = {
        let media_library = state.media_signal_watcher.data.borrow();
        let media = media_library
            .iter()
            .find(|media| media.id == media_id)
            .ok_or(StatusCode::NOT_FOUND.into_response())?;

        selections
            .into_iter()
            .map(|selection| {
                media
                    .get_media_paths(selection.episode_identifier())
                    .map(|media_paths| (selection, state.media_dir.join(&media_paths.media)))
            })
            .collect::<Option<Vec<(SubtitleSelection, PathBuf)>>>()
    }
    .ok_or(StatusCode::BAD_REQUEST)?;

    // 2. Download
    let futures = request_path_pairs
        .into_iter()
        .map(|(selection, media_path)| {
            let subtitle_id = *selection.subtitle_id();
            download_subtitle(
                media_path,
                selection,
                language_code.clone(),
                state.subtitle_signal_sender.clone(),
            )
            .map(move |future_result| future_result.map(|result| (subtitle_id, result)))
        });

    // 3. Check if any internal errors happened
    let result: axum::response::Result<Vec<(usize, Result<(), SubtitleDownloadError>)>> =
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
    selection: SubtitleSelection,
    language_code: LanguageCode,
    signal_sender: SubtitleSignalSender,
) -> axum::response::Result<Result<(), SubtitleDownloadError>> {
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

    let signal = SubtitleSignal::Download {
        media_path,
        result_sender,
        selection,
        language_code,
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
