use std::path::Path;

use domain::subtitles::Subtitle;

pub(super) async fn extract_subtitles(
    media_path: impl AsRef<Path>,
) -> crate::crawl::Result<impl Iterator<Item = Subtitle>> {
    Ok(std::iter::empty())
}

fn get_srt_language(srt_path: &impl AsRef<Path>) -> Option<&str> {
    todo!()
}
