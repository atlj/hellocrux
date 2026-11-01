use crate::crawl::{media_file::crawl_potential_media_file, subtitles::crawl_subtitles_dir};

use super::Result;
use std::path::Path;

pub(super) async fn crawl_movie_content(
    at: impl AsRef<Path>,
) -> Result<Option<(domain::MediaPaths, bool)>> {
    // 1. See if we have subtitles folder, if so generate a subtitles pool
    let subtitles = crawl_subtitles_dir(&at).await?;

    // 2. Get through all present files to see if one of them is a compatible movie file
    for entry in crate::dir::fully_read_dir(&at).await? {
        let path = entry.path();
        if let Some((paths, _, needs_to_be_prepared)) =
            crawl_potential_media_file(path, subtitles.as_deref(), None).await?
        {
            return Ok(Some((paths, needs_to_be_prepared)));
        }
    }

    Ok(None)
}
