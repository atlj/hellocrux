use crate::{language::LanguageCode, series::EpisodeIdentifier};

pub trait SubtitleItem {
    fn get_language(&self) -> &LanguageCode;
    fn get_title(&self) -> &str;
    fn get_download_count(&self) -> Option<usize>;
}

pub trait SubtitleProvider {
    type Item: SubtitleItem;
    type Error;

    fn search_subtitles(
        &self,
        title: &str,
        language: LanguageCode,
        episode: Option<EpisodeIdentifier>,
    ) -> impl Future<Output = Result<impl Iterator<Item = Self::Item>, Self::Error>>;

    fn download_subtitles(
        &self,
        item: &Self::Item,
    ) -> impl Future<Output = Result<String, Self::Error>>;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Subtitle {
    pub name: String,
    pub language_iso639_2t: String,
    pub path: String,
    /// A container such as mp4 that has a subtitle stream
    pub track_path: String,
}

// TODO consolidate some fields
#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct AddSubtitleForm {
    pub media_id: String,
    pub episode_identifier: Option<EpisodeIdentifier>,
    pub language_iso639: String,
    pub name: String,
    pub extension: String,
    pub file_contents: String,
}
