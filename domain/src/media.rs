use crate::{series::EpisodeIdentifier, subtitles::Subtitle};
use std::{collections::HashMap, path::Path};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Media {
    pub id: String,
    pub metadata: MediaMetaData,
    pub content: MediaContent,
}

impl Media {
    pub fn get_media_paths(
        &self,
        episode_identifier: Option<&EpisodeIdentifier>,
    ) -> Option<&MediaPaths> {
        match (&self.content, episode_identifier) {
            (MediaContent::Movie(paths), None) => Some(paths),
            (
                MediaContent::Series(hash_map),
                Some(EpisodeIdentifier {
                    season_no,
                    episode_no,
                }),
            ) => hash_map
                .get(season_no)
                .and_then(|season| season.get(episode_no)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum MediaContent {
    Movie(MediaPaths),
    Series(SeriesContents),
}

/// Episode no -> paths
pub type SeasonContents = HashMap<u32, MediaPaths>;
/// Season no -> season contents
pub type SeriesContents = HashMap<u32, SeasonContents>;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct MediaPaths {
    pub media: String,
    pub track_name: String,
    pub subtitles: Box<[Subtitle]>,
}

impl MediaPaths {
    pub fn add_prefix(&self, prefix: impl AsRef<Path>) -> Self {
        let media = prefix
            .as_ref()
            .join(&self.media)
            .to_string_lossy()
            .to_string();

        let subtitles = self
            .subtitles
            .iter()
            .map(|subtitle| Subtitle {
                path: prefix
                    .as_ref()
                    .join(&subtitle.path)
                    .to_string_lossy()
                    .to_string(),
                ..subtitle.clone()
            })
            .collect();

        Self {
            media,
            subtitles,
            track_name: self.track_name.clone(),
        }
    }

    pub fn strip_prefix(&self, prefix: impl AsRef<Path>) -> Option<Self> {
        let media = self
            .media
            .strip_prefix(prefix.as_ref().to_string_lossy().as_ref())?
            .trim_start_matches('/')
            .to_string();

        let subtitles = self
            .subtitles
            .iter()
            .map(|subtitle| {
                Some(Subtitle {
                    path: subtitle
                        .path
                        .strip_prefix(prefix.as_ref().to_string_lossy().as_ref())?
                        .trim_start_matches('/')
                        .to_string(),
                    ..subtitle.clone()
                })
            })
            .collect::<Option<Box<[_]>>>()?;

        Some(Self {
            subtitles,
            media,
            track_name: self.track_name.clone(),
        })
    }
}

impl MediaContent {
    pub fn add_prefix(mut self, prefix: impl AsRef<Path>) -> Self {
        match &mut self {
            MediaContent::Movie(media_paths) => {
                *media_paths = media_paths.add_prefix(prefix);
                self
            }
            MediaContent::Series(hash_map) => {
                for season in hash_map.values_mut() {
                    for episode in season.values_mut() {
                        *episode = episode.add_prefix(&prefix);
                    }
                }
                self
            }
        }
    }

    pub fn strip_prefix(mut self, prefix: impl AsRef<Path>) -> Option<Self> {
        match &mut self {
            MediaContent::Movie(movie_path) => {
                *movie_path = movie_path.strip_prefix(prefix)?;
                Some(self)
            }
            MediaContent::Series(hash_map) => {
                for season in hash_map.values_mut() {
                    for episode in season.values_mut() {
                        *episode = episode.strip_prefix(&prefix)?;
                    }
                }
                Some(self)
            }
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct MediaMetaData {
    pub thumbnail: String,
    pub title: String,
}
