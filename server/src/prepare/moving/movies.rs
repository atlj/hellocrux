use std::path::{Path, PathBuf};

use crate::prepare::{Error, Result};
use domain::MediaMetaData;
use tokio::io::AsyncWriteExt;

use super::sanitize_name_for_url;

/// Returns the resulting movie file's path
pub async fn generate_movie_media(
    media_dir: &Path,
    movie_file: &Path,
    metadata: &MediaMetaData,
) -> Result<PathBuf> {
    let target_dir = media_dir.join(sanitize_name_for_url(&metadata.title));

    // 1. Create destination dir
    {
        tokio::fs::create_dir_all(&target_dir)
            .await
            .map_err(|err| {
                Error::MoveError(
                    format!(
                        "Couldn't generate media dir at {}. Reason: {err}",
                        media_dir.display()
                    )
                    .into(),
                )
            })?;
    }

    // 2. Move movie file to destination
    {
        let extension = movie_file.extension().ok_or_else(|| {
            Error::MoveError(
                format!("Can't read movie file extension {}", movie_file.display()).into(),
            )
        })?;

        tokio::fs::rename(
            movie_file,
            target_dir.join(format!("movie-tbd.{}", extension.to_string_lossy())),
        )
        .await
        .map_err(|err| {
            Error::MoveError(
                format!(
                    "Couldn't move movie file from {} to {}. Reason: {err}",
                    movie_file.display(),
                    target_dir.display()
                )
                .into(),
            )
        })?;
    }

    // 3. Save metadata
    {
        let mut metadata_file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(target_dir.join("meta.json"))
            .await
            .map_err(|err| {
                Error::MoveError(format!("Can't open meta.json. Reason: {err}",).into())
            })?;

        let metadata_string = serde_json::to_string_pretty(metadata).map_err(|err| {
            Error::MoveError(format!("Can't serialize metadata into json. Reason: {err}").into())
        })?;

        metadata_file
            .write(metadata_string.as_bytes())
            .await
            .map_err(|err| {
                Error::MoveError(format!("Couldn't save metadata. Reason: {err}").into())
            })?;
    }

    Ok(target_dir)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use domain::MediaMetaData;

    use crate::prepare::moving::generate_movie_media;

    #[tokio::test]
    async fn test_generate_movie_media() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();

        tokio::fs::copy(
            test_data_path.join("test.mkv"),
            test_data_path.join("test_copy.mkv"),
        )
        .await
        .unwrap();

        let metadata = MediaMetaData {
            title: "My Movie".to_string(),
            thumbnail: "http://path.to/image".to_string(),
        };

        let _ = tokio::fs::remove_dir_all(test_data_path.join("tmp/generate_movie_media")).await;

        generate_movie_media(
            &test_data_path.join("tmp/generate_movie_media"),
            &test_data_path.join("test_copy.mkv"),
            &metadata,
        )
        .await
        .unwrap();
    }
}
