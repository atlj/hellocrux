use crate::{language::LanguageCode, series::EpisodeIdentifier};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SearchSubtitlesQuery {
    pub media_id: String,
    pub language_code: LanguageCode,
    pub season_no: Option<u32>,
    pub episode_no: Option<u32>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SubtitleRequest {
    pub episode_identifier: Option<EpisodeIdentifier>,
    pub subtitle_id: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SubtitleDownloadForm {
    pub media_id: String,
    pub requests: Box<[SubtitleRequest]>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Subtitle {
    pub language_iso639_2t: String,
    pub path: String,
    /// A container such as mp4 that has a subtitle stream
    pub track_path: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum SubtitleDownloadError {
    SubtitleAlreadyExists,
    DownloadQuotaReached,
    InternalFileSystemError,
    NonExistentSubtitle,
}

impl std::fmt::Display for SubtitleDownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for SubtitleDownloadError {}
