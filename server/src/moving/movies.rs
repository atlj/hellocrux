use std::path::{Path, PathBuf};

use super::{Error, Result};
use domain::MediaMetaData;

use super::sanitize_name_for_url;

/// Returns the resulting movie file's path
pub async fn generate_movie_media(
    media_dir: &Path,
    movie_file: &Path,
    metadata: &MediaMetaData,
) -> Result<PathBuf> {
    let target_dir = media_dir.join(sanitize_name_for_url(&metadata.title));
    // We want to avoid URL breaking names since files are hosted directly with their names
    let file_name = {
        let movie_file_stem = movie_file
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or(Error::NoFileName)?;
        let extension = movie_file.extension().ok_or(Error::NoExtension)?;

        let url_encoded_stem = domain::encode_decode::encode_url_safe(movie_file_stem);
        format!("{url_encoded_stem}.{}", extension.to_string_lossy())
    };

    // 1. Create destination dir
    tokio::fs::create_dir_all(&target_dir)
        .await
        .map_err(|err| Error::CantCreateDir {
            path: target_dir.to_path_buf(),
            inner: err,
        })?;

    // 2. Move movie file to destination
    let destination = target_dir.join(&file_name);
    tokio::fs::rename(movie_file, &destination)
        .await
        .map_err(|err| Error::CantMove {
            from: movie_file.to_path_buf(),
            to: destination,
            inner: err,
        })?;

    // 3. Save metadata
    super::metadata::save_metadata(&target_dir, metadata.clone()).await?;

    Ok(target_dir.join(&file_name))
}

#[cfg(test)]
mod tests {
    use domain::MediaMetaData;

    use crate::{moving::generate_movie_media, test_utils::fixtures_path};

    #[tokio::test]
    async fn test_generate_movie_media() {
        let tmp = tempfile::tempdir().unwrap();

        let src = fixtures_path().join("test.mkv");
        let working_copy = tmp.path().join("test_copy.mkv");
        tokio::fs::copy(&src, &working_copy).await.unwrap();

        let metadata = MediaMetaData {
            title: "My Movie".to_string(),
            thumbnail: "http://path.to/image".to_string(),
        };

        let output_dir = tmp.path().join("generate_movie_media");
        let movie_file_path = generate_movie_media(&output_dir, &working_copy, &metadata)
            .await
            .unwrap();

        dbg!(&movie_file_path);

        assert!(tokio::fs::try_exists(movie_file_path).await.unwrap());
    }
}
