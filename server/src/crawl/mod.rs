use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    path::{Path, PathBuf},
};

use domain::{Media, MediaContent};
use log::{error, warn};

use crate::crawl::metadata::read_metadata;

mod media_file;
mod metadata;
mod movie;
mod series;
mod subtitles;

pub(crate) async fn crawl_all_folders(
    path: impl AsRef<Path> + Display,
) -> (
    HashMap<String, Media>,
    Vec<(domain::MediaMetaData, domain::MediaIdentifier)>,
) {
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

        Some(async move { crawl_folder(path.to_string_lossy().as_ref()).await })
    });

    let media = futures::future::join_all(crawl_futures).await;
    let len = media.len();

    media.into_iter().fold(
        (HashMap::with_capacity(len), Vec::with_capacity(len)),
        |(mut media, mut to_prepare), result| {
            let Some((current_media, current_to_prepare)) = result else {
                return (media, to_prepare);
            };

            if let Some(current_media) = current_media {
                media.insert(current_media.id.clone(), current_media);
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
    // Files that need to be prepared
    Vec<(domain::MediaMetaData, domain::MediaIdentifier)>,
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
) -> Result<(
    Option<Media>,
    Vec<(domain::MediaMetaData, domain::MediaIdentifier)>,
)> {
    let id: String = path
        .as_ref()
        .components()
        .next_back()
        .map(|last| last.as_os_str().to_string_lossy().into())
        .ok_or_else(|| Error::CantReadDir(path.as_ref().into()))?;

    let metadata = read_metadata(&path).await?;
    let (content, to_prepare_without_metadata) = crawl_media_content(id.clone(), &path).await?;

    let to_prepare = to_prepare_without_metadata
        .into_iter()
        .map(|to_prepare_item| (metadata.clone(), to_prepare_item))
        // TODO get rid of a second collect, make to_prepare_without_metadata an iter instead of a
        // vec.
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
    id: String,
    path: impl AsRef<Path>,
) -> Result<(Option<MediaContent>, Vec<domain::MediaIdentifier>)> {
    // 1. Is it a movie?
    if let Some((movie_paths, needs_to_be_prepared)) = movie::crawl_movie_content(&path).await? {
        return Ok(match needs_to_be_prepared {
            false => (
                Some(MediaContent::Movie(movie_paths)),
                Vec::with_capacity(0),
            ),
            true => (
                None,
                vec![domain::MediaIdentifier::Movie {
                    id,
                    path: movie_paths,
                }],
            ),
        });
    };

    // 2. Then perhaps it's a TV series
    if let Some((seasons, to_prepare)) = series::crawl_series_content(&path).await? {
        return Ok((
            {
                if seasons.is_empty() {
                    None
                } else {
                    Some(MediaContent::Series(seasons))
                }
            },
            to_prepare
                .into_iter()
                .map(
                    |(episode_identifier, paths)| domain::MediaIdentifier::Series {
                        id: id.clone(),
                        path: paths,
                        episode: episode_identifier,
                    },
                )
                .collect(),
        ));
    }

    // 3. It was nothing
    Err(Error::NoMediaContent)
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
    use super::get_numeric_content;

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
    #[error("Can't check compatibility of media file. {0}")]
    CantCheckCompatibility(#[from] crate::prepare::Error),
    #[error("IO Error {0}")]
    FileSystem(#[from] std::io::Error),
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::CorruptedMetadata
    }
}
