pub mod encode_decode;
pub mod format;
pub mod language;
pub mod series;
pub mod subtitles;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use crate::{series::EpisodeIdentifier, subtitles::Subtitle};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaContent {
    Movie(MediaPaths),
    Series(SeriesContents),
}

/// Episode no -> paths
pub type SeasonContents = HashMap<u32, MediaPaths>;
/// Season no -> season contents
pub type SeriesContents = HashMap<u32, SeasonContents>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
                track_path: prefix
                    .as_ref()
                    .join(&subtitle.track_path)
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
                    track_path: subtitle
                        .track_path
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaMetaData {
    pub thumbnail: String,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Download {
    pub id: Box<str>,
    pub title: Box<str>,
    pub progress: f32,
    pub needs_file_mapping: bool,
    pub state: DownloadState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DownloadState {
    Paused,
    Failed,
    InProgress,
    Processing,
    Complete,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct DownloadForm {
    pub hash: Box<str>,
    pub metadata: MediaMetaData,
    pub is_series: bool,
}

pub enum MediaStream {
    Video,
    Audio,
}
