use domain::subtitles::Subtitle;
use either::Either;

use super::{Error, Result};
use std::path::Path;

pub(super) async fn try_extract_movie(
    media_path: impl AsRef<Path>,
) -> Result<Option<Either<domain::MediaPaths, domain::MediaIdentifier>>> {
    let mut read_dir = crate::dir::fully_read_dir(&media_path)
        .await
        .map_err(|_| Error::CantReadDir(media_path.as_ref().into()))?;

    let Some(movie_path) = read_dir.find_map(|entry| {
        let path = entry.path();
        if !domain::format::is_video_file(&path) {
            return None;
        }

        Some(path.to_string_lossy().to_string())
    }) else {
        return Ok(None);
    };

    let subtitles: Vec<Subtitle> = crate::crawl::subtitles::extract_subtitles(&media_path).await?;

    let file_stem = (movie_path.as_ref() as &Path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("Media file to have a proper stem");

    let track_name =
        domain::encode_decode::decode_url_safe(file_stem).unwrap_or_else(|_| file_stem.to_string());

    let media_paths = domain::MediaPaths {
        media: movie_path.clone(),
        subtitles,
        track_name,
    };

    if crate::prepare::needs_to_be_prepared(&movie_path)
        .await
        .map_err(|err| Error::CantCheckCompatibility(err))?
    {
        return Ok(Some(Either::Right(domain::MediaIdentifier::Movie {
            // Will be replaced (hopefully)
            id: "".to_string(),
            path: media_paths,
        })));
    }

    Ok(Some(Either::Left(media_paths)))
}
