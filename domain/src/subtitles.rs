use std::collections::HashMap;

use crate::{language::LanguageCode, series::EpisodeIdentifier};

/// The server expects this form for subtitle search requests
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SubtitleSearchForm {
    pub media_id: String,
    pub language_code: LanguageCode,
    pub episode_identifiers: Option<Vec<EpisodeIdentifier>>,
}

/// Returned by the search endpoint
pub type SubtitleSearchResponse = Vec<Vec<SubtitleDownloadOption<usize>>>;

/// The server expects this form for subtitle download requests
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SubtitleDownloadForm {
    pub media_id: String,
    pub language_code: LanguageCode,
    pub selections: Vec<SubtitleSelection>,
}

/// Returned by the download endpoint
pub type SubtitleDownloadResponse = HashMap<usize, Result<(), SubtitleDownloadError>>;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum SubtitleSelection {
    Series {
        subtitle_id: usize,
        episode_identifier: EpisodeIdentifier,
    },
    Movie {
        subtitle_id: usize,
    },
}

impl SubtitleSelection {
    pub fn subtitle_id(&self) -> &usize {
        match self {
            SubtitleSelection::Series { subtitle_id, .. } => subtitle_id,
            SubtitleSelection::Movie { subtitle_id } => subtitle_id,
        }
    }

    pub fn episode_identifier(&self) -> Option<&EpisodeIdentifier> {
        match self {
            SubtitleSelection::Series {
                episode_identifier, ..
            } => Some(episode_identifier),
            SubtitleSelection::Movie { .. } => None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Subtitle {
    pub language: LanguageCode,
    pub path: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum SubtitleDownloadError {
    SubtitleAlreadyExists,
    DownloadQuotaReached,
    InternalFileSystemError,
    NonExistentSubtitle,
}

pub trait SubtitleProvider {
    type SubtitleId: serde::Serialize + serde::de::DeserializeOwned;
    type Error: std::error::Error;

    fn search(
        &self,
        query: &str,
        language: LanguageCode,
        episode: Option<EpisodeIdentifier>,
    ) -> impl Future<
        Output = Result<
            impl Iterator<Item = SubtitleDownloadOption<Self::SubtitleId>>,
            Self::Error,
        >,
    >;

    fn download(&self, id: &Self::SubtitleId) -> impl Future<Output = Result<String, Self::Error>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubtitleDownloadOption<Id> {
    pub id: Id,
    pub title: String,
    pub download_count: usize,
    pub language: LanguageCode,
}

impl std::fmt::Display for SubtitleDownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for SubtitleDownloadError {}
