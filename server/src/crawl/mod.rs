use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
};

use domain::{Media, MediaContent, MediaMetaData};
use log::{error, warn};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

mod movie;
mod series;

pub(crate) async fn crawl_all_folders(path: impl AsRef<Path> + Display) -> Option<Box<[Media]>> {
    let read_dir = crate::dir::fully_read_dir(&path)
        .await
        .inspect_err(|err| error!("Media library at {path} isn't readable. Reason: {err}"))
        .ok()?;

    let crawl_futures = read_dir.map(|entry| async move {
        let path = entry.path();
        if path.is_file() {
            let result: Option<Media> = None;
            return result;
        }

        crawl_folder(path.to_string_lossy().as_ref()).await
    });

    let result = futures::future::join_all(crawl_futures)
        .await
        .into_iter()
        .flatten()
        .collect();

    Some(result)
}

pub(crate) async fn crawl_folder(path: impl AsRef<Path> + Display) -> Option<Media> {
    let mut result = try_extract_media(&path)
        .await
        .inspect_err(|err| warn!("Couldn't extract media content from {path}. Reason: {err}"))
        .ok()?;

    let stripped_content = path
        .as_ref()
        .parent()
        .and_then(|parent| result.content.strip_prefix(parent));

    if stripped_content.is_none() {
        warn!(
            "Couldn't strip prefix of media named {}. Ignoring it.",
            &result.metadata.title
        );
    }

    result.content = stripped_content?;

    Some(result)
}

async fn try_extract_media(path: impl AsRef<Path>) -> Result<Media> {
    let metadata = try_extract_metadata(&path).await?;
    let content = try_extract_media_content(&path).await?;

    Ok(Media {
        id: metadata.title.clone(),
        metadata,
        content,
    })
}

async fn try_extract_media_content(path: impl AsRef<Path>) -> Result<MediaContent> {
    if let Some(movie_path) = movie::try_extract_movie_paths(&path).await? {
        return Ok(MediaContent::Movie(movie_path));
    }

    if let Some(series) = series::try_extract_series(&path).await? {
        return Ok(MediaContent::Series(series));
    }

    Err(Error::NoMediaContent)
}

async fn try_extract_metadata(path: impl AsRef<Path>) -> Result<MediaMetaData> {
    let metadata_string = {
        let mut metadata_file = OpenOptions::new()
            .read(true)
            .open(path.as_ref().join("meta.json"))
            .await
            .map_err(|_| Error::NoMetadata)?;

        let mut string = String::new();
        metadata_file
            .read_to_string(&mut string)
            .await
            .map_err(|_| Error::CorruptedMetadata)?;
        string
    };

    Ok(serde_json::from_str(&metadata_string)?)
}

fn get_numeric_content(string: &str) -> Option<u32> {
    string
        .chars()
        .skip_while(|char| !char.is_ascii_digit())
        .map_while(|char| char.to_digit(10))
        .fold(None, |acc, digit| Some(acc.unwrap_or(0) * 10 + digit))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{get_numeric_content, try_extract_metadata};

    #[tokio::test]
    async fn test_extract_metadata() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let path = test_data_path.join("crawl/example_movie");
        try_extract_metadata(path).await.unwrap();
    }

    #[test]
    fn can_get_numeric_content() {
        assert_eq!(get_numeric_content("1Ambush.mov"), Some(1));
        assert_eq!(get_numeric_content("176hey.exe"), Some(176));
        assert_eq!(get_numeric_content("02Ambush.mov"), Some(2));
        assert_eq!(get_numeric_content("22ey17.exe"), Some(22));
        assert_eq!(get_numeric_content("eyslkvjsdlkj03k.exe"), Some(3));
        assert_eq!(get_numeric_content("1"), Some(1));
        assert_eq!(get_numeric_content("Ambush.mov"), None);
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoMetadata,
    CorruptedMetadata,
    CantReadMetadata,
    NoMediaContent,
    CantReadDir(PathBuf),
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::CorruptedMetadata
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoMetadata => f.write_str("No metadata found"),
            Error::CorruptedMetadata => f.write_str("Metadata was corrupted"),
            Error::CantReadMetadata => f.write_str("Couldn't read metadata"),
            Error::NoMediaContent => f.write_str("No media content"),
            Error::CantReadDir(path_buf) => write!(f, "Couldn't read {:#?}", path_buf),
        }
    }
}

impl std::error::Error for Error {}
