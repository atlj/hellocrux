use std::path::Path;

use axum::{Json, extract, http::StatusCode, response::IntoResponse};
use domain::{series::EpisodeIdentifier, subtitles::SearchSubtitlesQuery};
use log::warn;
use subtitles::SubtitleProvider;

use crate::State;

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

    // 1. Find the media file name
    let media_file_name: String = {
        // 1a: Get media from library
        let media_data = state.media_signal_watcher.data.borrow();
        let media = media_data
            .iter()
            .find(|media| media.id == query.media_id)
            .ok_or(StatusCode::NOT_FOUND.into_response())?;

        let media_path: &Path = match (&media.content, &episode_identifier) {
            (
                domain::MediaContent::Series(series),
                Some(EpisodeIdentifier {
                    episode_no,
                    season_no,
                }),
            ) => {
                let media_path = series
                    .get(season_no)
                    .and_then(|season| season.get(episode_no))
                    .ok_or(StatusCode::BAD_REQUEST)?
                    .media
                    .as_ref();

                Ok(media_path)
            }
            (domain::MediaContent::Movie(media_paths), None) => Ok(media_paths.media.as_ref()),
            _ => Err(StatusCode::BAD_REQUEST.into_response()),
        }?;

        let file_stem = media_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("File stem to be valid");

        // 1b: Decode the file stem using b64
        domain::encode_decode::decode_url_safe(file_stem).unwrap_or_else(|_| {
                        warn!("While searching for subtitles of {} episode {:#?}, couldn't decode the file stem using base64 ({file_stem}). Returning the file stem directly.", media.metadata.title, &episode_identifier);
                        file_stem.to_string()
                    })
    };

    let result = state
        .subtitle_provider
        .search(&media_file_name, query.language_code, episode_identifier)
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
