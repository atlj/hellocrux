#[derive(Debug)]
pub struct SubtitleProvider {}

impl SubtitleProvider {
    pub fn new() -> Self {
        todo!()
    }
}

impl domain::subtitles::SubtitleProvider for SubtitleProvider {
    type Identifier = String;
    type Error = ();

    fn search_subtitles<F>(
        &self,
        title: &str,
        language: domain::language::LanguageCode,
        episode: Option<domain::series::EpisodeIdentifier>,
    ) -> F
    where
        F: Future<Output = Result<Box<[Self::Identifier]>, Self::Error>>,
    {
        todo!()
    }

    fn download_subtitles<F>(&self, identifier: &Self::Identifier) -> F
    where
        F: Future<Output = Result<String, Self::Error>>,
    {
        todo!()
    }
}
