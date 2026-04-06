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
pub enum MediaIdentifier {
    Movie {
        id: String,
        path: MediaPaths,
    },
    Series {
        id: String,
        path: MediaPaths,
        episode: EpisodeIdentifier,
    },
}

impl MediaIdentifier {
    pub fn id(&self) -> &str {
        match self {
            MediaIdentifier::Movie { id, .. } => id,
            MediaIdentifier::Series { id, .. } => id,
        }
    }

    pub fn path(&self) -> &MediaPaths {
        match self {
            MediaIdentifier::Movie { path, .. } => path,
            MediaIdentifier::Series { path, .. } => path,
        }
    }

    pub fn with_path(self, path: MediaPaths) -> Self {
        match self {
            MediaIdentifier::Movie { id, .. } => MediaIdentifier::Movie { id, path },
            MediaIdentifier::Series { id, episode, .. } => {
                MediaIdentifier::Series { id, episode, path }
            }
        }
    }

    pub fn with_id(self, id: String) -> Self {
        match self {
            MediaIdentifier::Movie { path, .. } => MediaIdentifier::Movie { id, path },
            MediaIdentifier::Series { episode, path, .. } => {
                MediaIdentifier::Series { id, episode, path }
            }
        }
    }

    pub fn strip_prefix(self, prefix: impl AsRef<Path>) -> Option<Self> {
        let path = self.path();
        let stripped_path = path.strip_prefix(&prefix)?;
        Some(self.with_path(stripped_path))
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
    pub subtitles: Vec<Subtitle>,
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
            .collect::<Option<Vec<_>>>()?;

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

pub const DEFAULT_CONTAINER_FORMAT: &str = "mp4";
pub const DEFAULT_VIDEO_CODEC: &str = "hevc";
pub const DEFAULT_AUDIO_CODEC: &str = "aac";

pub fn is_container_compatible(container: &str) -> bool {
    matches!(container, "mp4")
}

pub fn is_video_codec_compatible(video_codec: &str) -> bool {
    matches!(video_codec, "hevc" | "hvc1" | "h264")
}

pub fn is_audio_codec_compatible(audio_codec: &str) -> bool {
    matches!(audio_codec, "ac3" | "aac" | "eac3")
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct MediaMetaData {
    pub thumbnail: String,
    pub title: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum Track {
    Video {
        id: usize,
        codec: String,
        duration: Option<std::time::Duration>,
    },
    Audio {
        id: usize,
        codec: String,
        duration: Option<std::time::Duration>,
        language: Option<crate::language::LanguageCode>,
    },
    Subtitle {
        id: usize,
        language: Option<crate::language::LanguageCode>,
        external_id: Option<String>,
    },
}

impl Track {
    pub fn id(&self) -> &usize {
        match self {
            Track::Video { id, .. } => id,
            Track::Audio { id, .. } => id,
            Track::Subtitle { id, .. } => id,
        }
    }

    pub fn is_codec_compatible(&self) -> bool {
        match self {
            Track::Video { codec, .. } => is_video_codec_compatible(codec),
            Track::Audio { codec, .. } => is_audio_codec_compatible(codec),
            Track::Subtitle { .. } => true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TrackSelectionItem {
    pub media: MediaIdentifier,
    pub tracks: Vec<Track>,
}

impl TrackSelectionItem {
    fn with_media(self, media: MediaIdentifier) -> Self {
        Self { media, ..self }
    }

    pub fn strip_prefix(self, prefix: impl AsRef<Path>) -> Option<Self> {
        let stripped_media = self.media.clone().strip_prefix(prefix)?;
        Some(self.with_media(stripped_media))
    }
}
