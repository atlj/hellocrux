pub mod format;
pub mod series;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Media {
    pub id: String,
    pub metadata: MediaMetaData,
    pub content: MediaContent,
}

pub enum MediaStream {
    Video,
    Audio,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaContent {
    Movie(String),
    Series(SeriesContents),
}

impl MediaContent {
    pub fn strip_prefix(mut self, prefix: impl AsRef<Path>) -> Option<Self> {
        match &mut self {
            MediaContent::Movie(movie_path) => movie_path
                .strip_prefix(prefix.as_ref().to_string_lossy().as_ref())
                .map(|stripped_path| {
                    Self::Movie(stripped_path.trim_start_matches('/').to_string())
                }),
            MediaContent::Series(hash_map) => {
                for season in hash_map.values_mut() {
                    let prefix_removed = season
                        .values()
                        .map(|path| {
                            path.strip_prefix(prefix.as_ref().to_string_lossy().as_ref())
                                .map(|str| str.trim_start_matches('/').to_string())
                        })
                        .collect::<Option<Box<[_]>>>()?;

                    season
                        .values_mut()
                        .zip(prefix_removed)
                        .for_each(|(value, new_path)| {
                            *value = new_path;
                        });
                }
                Some(self)
            }
        }
    }
}

/// Episode no -> path
pub type SeasonContents = HashMap<u32, String>;
/// Season no -> season contents
pub type SeriesContents = HashMap<u32, SeasonContents>;

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
