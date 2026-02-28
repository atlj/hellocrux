use domain::{language::LanguageCode, series::EpisodeIdentifier};
pub struct SubtitleDownloadOption<Id>
where
    Id: serde::Serialize + serde::de::DeserializeOwned,
{
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
        Output = impl Iterator<Item = Result<SubtitleDownloadOption<Self::SubtitleId>, Self::Error>>,
    >;

    fn download(
        &self,
        item: &SubtitleDownloadOption<Self::SubtitleId>,
    ) -> impl Future<Output = Result<String, Self::Error>>;
}
