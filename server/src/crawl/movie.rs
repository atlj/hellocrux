use super::{Error, Result};
use std::path::Path;

pub(super) async fn try_extract_movie_paths(
    path: impl AsRef<Path>,
) -> Result<Option<domain::MediaPaths>> {
    let mut read_dir = crate::dir::fully_read_dir(&path)
        .await
        .map_err(|_| Error::CantReadDir(path.as_ref().into()))?;

    let result = read_dir.find_map(|entry| {
        let path = entry.path();
        if !domain::format::is_supported_video_file(&path) {
            return None;
        }

        Some(path.to_string_lossy().into())
    });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::crawl::movie::try_extract_movie_paths;

    #[tokio::test]
    async fn extract_movie_path() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let path = test_data_path.join("crawl/example_movie");
        assert!(
            try_extract_movie_paths(path)
                .await
                .unwrap()
                .unwrap()
                .media
                .contains("hey.mp4")
        );
    }
}
