use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    path::{Path, PathBuf},
};

use domain::{Media, MediaContent, MediaMetaData};
use log::{error, warn};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

mod movie;
mod series;
mod subtitles;

pub(crate) async fn crawl_all_folders(
    path: impl AsRef<Path> + Display,
) -> (HashMap<String, Media>, Vec<domain::MediaIdentifier>) {
    let Some(read_dir) = crate::dir::fully_read_dir(&path)
        .await
        .inspect_err(|err| error!("Media library at {path} isn't readable. Reason: {err}"))
        .ok()
    else {
        return (HashMap::new(), Vec::new());
    };

    let crawl_futures = read_dir.flat_map(|entry| {
        let path = entry.path();
        if path.is_file() {
            return None;
        }

        let file_stem = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())?;

        Some(async move {
            crawl_folder(path.to_string_lossy().as_ref())
                .await
                .map(|media| (file_stem, media))
        })
    });

    let media = futures::future::join_all(crawl_futures).await;
    let len = media.len();

    media.into_iter().fold(
        (HashMap::with_capacity(len), Vec::with_capacity(len)),
        |(mut media, mut to_prepare), result| {
            let Some((id, (current_media, current_to_prepare))) = result else {
                return (media, to_prepare);
            };

            if let Some(current_media) = current_media {
                media.insert(id, current_media);
            }

            to_prepare.extend(current_to_prepare);

            (media, to_prepare)
        },
    )
}

pub(crate) async fn crawl_folder(
    path: impl AsRef<Path> + Display,
) -> Option<(
    Option<Media>,
    // Movies that need to be prepared
    Vec<domain::MediaIdentifier>,
)> {
    let (media, to_prepare) = crawl_media(&path)
        .await
        .inspect_err(|err| warn!("Couldn't extract media content from {path}. Reason: {err}"))
        .ok()?;

    let media = media.and_then(|mut media| {
        let stripped_content = path
            .as_ref()
            .parent()
            .and_then(|parent| media.content.strip_prefix(parent));

        if stripped_content.is_none() {
            warn!(
                "Couldn't strip prefix of media named {}. Ignoring it.",
                &media.metadata.title
            );
        }

        media.content = stripped_content?;

        Some(media)
    });

    Some((media, to_prepare))
}

async fn crawl_media(
    path: impl AsRef<Path>,
) -> Result<(Option<Media>, Vec<domain::MediaIdentifier>)> {
    let id: String = path
        .as_ref()
        .components()
        .next_back()
        .map(|last| last.as_os_str().to_string_lossy().into())
        .ok_or_else(|| Error::CantReadDir(path.as_ref().into()))?;
    let metadata = crawl_metadata(&path).await?;
    let (content, to_prepare_improper_ids) = crawl_media_content(&path).await?;
    let to_prepare = to_prepare_improper_ids
        .into_iter()
        .map(|identifier| identifier.with_id(id.clone()))
        .collect();

    let result = (
        content.map(|content| Media {
            id,
            metadata,
            content,
        }),
        to_prepare,
    );

    Ok(result)
}

async fn crawl_media_content(
    path: impl AsRef<Path>,
) -> Result<(Option<MediaContent>, Vec<domain::MediaIdentifier>)> {
    if let Some(movie) = movie::crawl_movie(&path).await? {
        let result = match movie {
            either::Either::Left(movie_path) => (Some(MediaContent::Movie(movie_path)), Vec::new()),
            either::Either::Right(to_prepare) => (None, vec![to_prepare]),
        };

        return Ok(result);
    }

    if let Some((seasons, to_prepare)) = series::crawl_series(&path).await? {
        return Ok(((seasons.map(MediaContent::Series)), to_prepare));
    }

    Err(Error::NoMediaContent)
}

async fn crawl_metadata(path: impl AsRef<Path>) -> Result<MediaMetaData> {
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
    use super::{crawl_metadata, get_numeric_content};

    use crate::test_utils::fixtures_path;

    #[tokio::test]
    async fn test_extract_metadata() {
        let path = fixtures_path().join("crawl/example_movie");
        crawl_metadata(path).await.unwrap();
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No metadata found")]
    NoMetadata,
    #[error("Metadata was corrupted")]
    CorruptedMetadata,
    #[error("Couldn't read metadata file")]
    CantReadMetadata,
    #[error("No media content")]
    NoMediaContent,
    #[error("Couldn't read dir {0:#?}")]
    CantReadDir(PathBuf),
    #[error("Couldn't convert subtitle to .mp4 file. Reason: {0}")]
    CantConvertSubtitle(crate::ffmpeg::Error),
    #[error("Can't check compatibility of media file. {0}")]
    CantCheckCompatibility(crate::prepare::Error),
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::CorruptedMetadata
    }
}

impl From<crate::ffmpeg::Error> for Error {
    fn from(value: crate::ffmpeg::Error) -> Self {
        Self::CantConvertSubtitle(value)
    }
}
