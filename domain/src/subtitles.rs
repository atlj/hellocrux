use crate::language::LanguageCode;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SearchSubtitlesQuery {
    pub media_id: String,
    pub language_code: LanguageCode,
    pub season_no: Option<u32>,
    pub episode_no: Option<u32>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Subtitle {
    pub language_iso639_2t: String,
    pub path: String,
    /// A container such as mp4 that has a subtitle stream
    pub track_path: String,
}
