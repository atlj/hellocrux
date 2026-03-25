use crate::MediaMetaData;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Download {
    pub id: Box<str>,
    pub title: Box<str>,
    pub progress: f32,
    pub needs_file_mapping: bool,
    pub state: DownloadState,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
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
