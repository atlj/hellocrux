use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct OpenSubtitlesResponse<T> {
    pub total_pages: usize,
    pub total_count: usize,
    pub per_page: usize,
    pub page: usize,
    pub data: Box<[T]>,
}

pub(super) type OpenSubtitlesSubtitleResponse = OpenSubtitlesResponse<OpenSubtitlesSubtitle>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct OpenSubtitlesSubtitle {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub attributes: Attributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub subtitle_id: String,
    pub language: String,
    pub download_count: usize,
    pub new_download_count: usize,
    pub hearing_impaired: bool,
    pub hd: bool,
    pub fps: f64,
    pub votes: usize,
    pub ratings: f64,
    pub from_trusted: bool,
    pub foreign_parts_only: bool,
    pub upload_date: String,
    pub ai_translated: bool,
    pub nb_cd: usize,
    pub slug: String,
    pub machine_translated: bool,
    pub release: String,
    pub comments: Option<String>,
    pub legacy_subtitle_id: Option<u64>,
    pub legacy_uploader_id: Option<u64>,
    pub uploader: Uploader,
    pub feature_details: FeatureDetails,
    pub url: String,
    pub related_links: Box<[RelatedLink]>,
    pub files: Box<[File]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Uploader {
    #[serde(rename = "uploader_id")]
    pub id: Option<u64>,
    pub name: String,
    pub rank: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct FeatureDetails {
    pub feature_id: u64,
    pub feature_type: String,
    pub year: usize,
    pub title: String,
    pub movie_name: String,
    pub imdb_id: u64,
    pub tmdb_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RelatedLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct File {
    pub file_id: usize,
    pub cd_number: usize,
    pub file_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct DownloadForm {
    pub file_id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct DownloadResponse {
    pub link: String,
    pub file_name: String,
    pub requests: u32,
    pub remaining: u32,
    pub message: String,
    pub reset_time: String,
    pub reset_time_utc: String,
}
