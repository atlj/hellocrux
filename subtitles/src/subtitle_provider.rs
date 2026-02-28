use domain::{language::LanguageCode, series::EpisodeIdentifier};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubtitleDownloadOption<Id> {
    pub id: Id,
    pub title: String,
    pub download_count: usize,
    pub language: LanguageCode,
}

pub trait SubtitleProvider {
    type SubtitleId: serde::Serialize + serde::de::DeserializeOwned;
    type Error: std::error::Error;

    fn search(
        &self,
        title: &str,
        language: LanguageCode,
        episode: Option<EpisodeIdentifier>,
    ) -> impl Future<
        Output = Result<
            impl Iterator<Item = SubtitleDownloadOption<Self::SubtitleId>>,
            Self::Error,
        >,
    >;

    fn download(
        &self,
        item: &SubtitleDownloadOption<Self::SubtitleId>,
    ) -> impl Future<Output = Result<String, Self::Error>>;
}
