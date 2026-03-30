use domain::subtitles::Subtitle;

use super::{Error, Result};
use log::error;
use std::path::Path;

pub(super) async fn try_extract_movie(
    media_path: impl AsRef<Path>,
) -> Result<Option<domain::MediaPaths>> {
    let mut read_dir = crate::dir::fully_read_dir(&media_path)
        .await
        .map_err(|_| Error::CantReadDir(media_path.as_ref().into()))?;

    let Some(media) = read_dir.find_map(|entry| {
        let path = entry.path();
        if !domain::format::is_supported_video_file(&path) {
            return None;
        }

        Some(path.to_string_lossy().to_string())
    }) else {
        return Ok(None);
    };

    let subtitles: Vec<Subtitle> = crate::crawl::subtitles::extract_subtitles(&media_path).await?;

    let movie_file_path: &Path = media.as_ref();

    embed_movie_subtitles(&movie_file_path, &subtitles).await;

    let file_stem = movie_file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("Media file to have a proper stem");
    let track_name =
        domain::encode_decode::decode_url_safe(file_stem).unwrap_or_else(|_| file_stem.to_string());

    Ok(Some(domain::MediaPaths {
        media,
        subtitles,
        track_name,
    }))
}

async fn embed_movie_subtitles(
    media_path: impl AsRef<Path> + std::fmt::Debug,
    subtitles: &[Subtitle],
) {
    let Some(file_name) = media_path
        .as_ref()
        .file_stem()
        .and_then(|stem| stem.to_str())
    else {
        error!("Media path has no file stem: {media_path:#?}",);
        return;
    };

    let Some(extension) = media_path
        .as_ref()
        .extension()
        .and_then(|stem| stem.to_str())
    else {
        error!("Media path has no extension: {media_path:#?}",);
        return;
    };

    let Ok(temp_dir) = tempfile::tempdir() else {
        error!("Couldn't create a temp dir to embed subs for {media_path:#?}. Skipping this step.",);
        return;
    };

    let temp_file_name = format!("{file_name}-tmp.{extension}");
    let temp_file = temp_dir.path().join(temp_file_name);

    match crate::crawl::subtitles::embed_subtitles_if_missing(&media_path, &temp_file, &subtitles)
        .await
    {
        Ok(did_embed) => {
            if !did_embed {
                return;
            }

            if let Err(error) = tokio::fs::copy(&temp_file, &media_path).await {
                error!(
                    "Couldn't move subtitle embedded file from {temp_file:#?} to {media_path:#?}. Reason: {error}"
                );
            }
        }
        Err(error) => {
            error!("Couldn't embed subtitles for {media_path:#?}. Reason: {error}")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crawl::movie::try_extract_movie;
    use crate::test_utils::fixtures_path;

    #[tokio::test]
    async fn extract_movie_path() {
        let path = fixtures_path().join("crawl/example_movie");
        let result = try_extract_movie(&path).await.unwrap().unwrap();
        assert!(result.media.contains("hey.mp4"));
        let subtitles = result.subtitles.first().unwrap();
        assert!(subtitles.path.contains("engSubs.vtt"));
        assert_eq!(subtitles.language, domain::language::LanguageCode::English);
    }
}
