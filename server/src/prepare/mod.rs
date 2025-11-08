use std::path::{Path, PathBuf};

use domain::MediaMetaData;

mod convert;
mod moving;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    ConvertError(Box<str>),
    MoveError(Box<str>),
    PrepareError(Box<str>),
}

pub async fn prepare_movie(
    media_dir: &Path,
    metadata: &MediaMetaData,
    source_dir: &Path,
) -> Result<()> {
    todo!()
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
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "mp4" | "mov" | "mkv" | "ts" => true,
            _ => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::prepare::find_movie_file;

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
}
