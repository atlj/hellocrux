use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use domain::MediaMetaData;
use log::info;

mod convert;
mod moving;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    ConvertError(Box<str>),
    MoveError(Box<str>),
    PrepareError(Box<str>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Error::ConvertError(message) => message,
            Error::MoveError(message) => message,
            Error::PrepareError(message) => message,
        })
    }
}

pub async fn prepare_movie(
    media_dir: &Path,
    metadata: &MediaMetaData,
    source_dir: &Path,
) -> Result<()> {
    // TODO consider using mappings for movies.
    // 1. Find movie file
    let movie_file = find_movie_file(source_dir)
        .await?
        .ok_or(Error::PrepareError(
            format!("No movie file found at {}", source_dir.display()).into(),
        ))?;

    // 2. Move movie media and generate metadata
    let target_dir = moving::generate_movie_media(media_dir, &movie_file, metadata).await?;

    // 3. Convert if needed
    {
        let moved_file = target_dir.join(
            movie_file
                .file_name()
                .expect("File with no filename detected"),
        );

        if convert::should_convert(&moved_file) {
            info!(
                "Media file with path {} is going to be converted.",
                moved_file.display()
            );

            convert::convert_file_to_mp4(&moved_file, &moved_file.with_extension("mp4")).await?;

            // 3a. Delete old file
            tokio::fs::remove_file(&moved_file).await.map_err(|err| {
            Error::PrepareError(
                format!(
                    "Converted a movie file but couldn't delete the source file at {}. Reason: {err}",
                    movie_file.display()
                )
                .into(),
            )
        })?;
        }
    }

    Ok(())
}

async fn find_movie_file(source_dir: &Path) -> Result<Option<PathBuf>> {
    let mut read_dir = tokio::fs::read_dir(source_dir).await.map_err(|err| {
        Error::PrepareError(
            format!(
                "Can't read source dir at {}. Reason: {err}",
                source_dir.display()
            )
            .into(),
        )
    })?;

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();

        if path.is_dir() {
            if let Some(sub_dir_media_path) = Box::pin(find_movie_file(&path)).await? {
                return Ok(Some(sub_dir_media_path));
            }
        }

        if check_if_video_file(&path) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn check_if_video_file(path: &Path) -> bool {
    match path.extension() {
        None => false,
        Some(extension) => matches!(
            extension.to_string_lossy().as_ref(),
            "mp4" | "mov" | "mkv" | "ts",
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::{self, Read},
        path::{Path, PathBuf},
    };

    use domain::MediaMetaData;

    use crate::prepare::{find_movie_file, prepare_movie};

    #[tokio::test]
    async fn test_prepare_movie() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();

        copy_dir_all(
            test_data_path.join("prepare"),
            test_data_path.join("tmp/prepare_source"),
        )
        .unwrap();

        let metadata = MediaMetaData {
            title: "Jellyfish".to_string(),
            thumbnail: "https://some-link".to_string(),
        };

        prepare_movie(
            &test_data_path.join("tmp/prepare"),
            &metadata,
            &test_data_path.join("tmp/prepare_source"),
        )
        .await
        .unwrap();

        tokio::fs::try_exists(&test_data_path.join("tmp/prepare/Jellyfish/movie.mp4"))
            .await
            .unwrap();

        let meta_file_contents = {
            let mut meta_file = OpenOptions::new()
                .read(true)
                .open(test_data_path.join("tmp/prepare/Jellyfish/meta.json"))
                .unwrap();
            let mut string = String::new();
            meta_file.read_to_string(&mut string).unwrap();
            string
        };

        let saved_metadata: MediaMetaData = serde_json::from_str(&meta_file_contents).unwrap();

        assert_eq!(metadata, saved_metadata);
    }

    #[tokio::test]
    async fn test_find_movie_file() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();

        assert_eq!(
            find_movie_file(&test_data_path.join("check_video_file/mkv"))
                .await
                .unwrap()
                .unwrap()
                .file_name()
                .unwrap(),
            "file.mkv"
        );

        assert_eq!(
            find_movie_file(&test_data_path.join("check_video_file/mov"))
                .await
                .unwrap()
                .unwrap()
                .file_name()
                .unwrap(),
            "hey.mov"
        );

        assert_eq!(
            find_movie_file(&test_data_path.join("check_video_file/mp4"))
                .await
                .unwrap()
                .unwrap()
                .file_name()
                .unwrap(),
            "file.mp4"
        );

        assert_eq!(
            find_movie_file(&test_data_path.join("check_video_file/nested"))
                .await
                .unwrap()
                .unwrap()
                .file_name()
                .unwrap(),
            "imavideo.mp4"
        );

        assert!(
            find_movie_file(&test_data_path.join("check_video_file/none"))
                .await
                .unwrap()
                .is_none()
        );
    }

    fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}
