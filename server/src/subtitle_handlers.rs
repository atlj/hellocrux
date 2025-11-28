use std::path::PathBuf;

use axum::{extract, http::StatusCode};
use domain::LanguageCode;
use regex::Regex;
use tokio::io::AsyncWriteExt;

pub async fn add_subtitle(
    extract::State(state): crate::State,
    extract::Json(domain::AddSubtitleForm {
        media_id,
        episode_identifier,
        extension,
        name,
        language_iso639,
        file_contents,
    }): extract::Json<domain::AddSubtitleForm>,
) -> axum::response::Result<()> {
    if !is_path_safe(&media_id) || 
        // TODO check if this is in a certain set.
        !is_path_safe(&extension) || !is_path_safe(&name) {
        return Err(StatusCode::FORBIDDEN.into());
    }

    let language_code =
        LanguageCode::try_from(language_iso639.as_str()).map_err(|_| StatusCode::BAD_REQUEST)?;

    let media_path = match episode_identifier {
        Some(domain::series::EpisodeIdentifier { season_no, .. }) => {
            state.media_dir.join(&media_id).join(season_no.to_string())
        }
        None => state.media_dir.join(&media_id),
    };

    if !tokio::fs::try_exists(&media_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::NOT_FOUND.into());
    }

    let subtitles_path = media_path.join("subtitles");
    tokio::fs::create_dir_all(&subtitles_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let save_path: PathBuf = {
        let mut file_name: PathBuf = match episode_identifier {
            Some(domain::series::EpisodeIdentifier { episode_no, .. }) => {
                format!("{}{}{}", episode_no, language_code.to_iso639_2t(), name)
            }

            None => format!("{}{}", language_code.to_iso639_2t(), name),
        }
        .into();
        file_name.set_extension(extension);
        subtitles_path.join(file_name)
    };

    if tokio::fs::try_exists(&save_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::FORBIDDEN.into());
    }

    let file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&save_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut writer = tokio::io::BufWriter::new(file);
    writer
        .write(file_contents.as_bytes())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    writer.flush().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .media_signal_watcher
        .signal_sender
        .send(crate::service::media::MediaSignal::CrawlPartial { media_id })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;


    Ok(())
}

fn is_path_safe(name: &str) -> bool {
    // Only allow letters, numbers, underscores, and hyphens
    // ^ and $ ensure the entire string matches
    let allowed = Regex::new(r"^[a-zA-Z0-9_-]{1,64}$").unwrap();

    allowed.is_match(name)
}
