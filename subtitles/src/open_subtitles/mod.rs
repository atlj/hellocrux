mod dto;

use std::{str::FromStr, sync::LazyLock};

use domain::series::EpisodeIdentifier;
use log::info;

use crate::{
    SubtitleProvider,
    open_subtitles::dto::{
        DownloadForm, DownloadResponse, OpenSubtitlesError, OpenSubtitlesSubtitleResponse,
    },
};

const API_KEY: &str = include_str!("../../../open_subtitles_api_key");
static OPEN_SUBTITLES_BASE_URL: LazyLock<reqwest::Url> = LazyLock::new(|| {
    reqwest::Url::parse("https://api.opensubtitles.com/api/v1/")
        .expect("Open Subtitles base url should be valid")
});
static DEFAULT_HEADERS: LazyLock<reqwest::header::HeaderMap> = LazyLock::new(|| {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        "Streamy v0.0.1".parse().unwrap(),
    );
    headers.insert("Api-Key", API_KEY.trim_end().parse().unwrap());
    headers.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());
    headers
});

#[derive(Debug, Clone)]
pub struct OpenSubtitlesClient {
    http_client: reqwest::Client,
}

impl Default for OpenSubtitlesClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenSubtitlesClient {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    fn check_api_error(api_response_str: &str) -> Result<()> {
        if let Ok(error) = serde_json::from_str::<OpenSubtitlesError>(api_response_str)
            && (error.status.is_some() || error.error.is_some())
        {
            return Err(error.into());
        }

        Ok(())
    }
}

impl SubtitleProvider for OpenSubtitlesClient {
    type SubtitleId = usize;
    type Error = Error;

    async fn search(
        &self,
        query: &str,
        language: domain::language::LanguageCode,
        episode: Option<domain::series::EpisodeIdentifier>,
    ) -> std::result::Result<
        impl Iterator<Item = crate::SubtitleDownloadOption<Self::SubtitleId>>,
        Self::Error,
    > {
        let search_url = {
            let mut url = OPEN_SUBTITLES_BASE_URL
                .join("subtitles")
                .expect("search URL to be valid");

            let encoded_title =
                percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC);

            let query_string = match episode {
                // Series
                Some(EpisodeIdentifier {
                    season_no,
                    episode_no,
                }) => [
                    "ai_translated=exclude",
                    &format!("episode_number={episode_no}"),
                    &format!("languages={}", language.to_iso639_1()),
                    "order_by=attributes%2Edownload_count",
                    &format!("query={encoded_title}"),
                    &format!("season_number={season_no}"),
                    "type=episode",
                ]
                .join("&"),
                // Movie
                None => [
                    "ai_translated=exclude",
                    &format!("languages={}", language.to_iso639_1()),
                    "order_by=attributes%2Edownload_count",
                    &format!("query={encoded_title}"),
                    "type=movie",
                ]
                .join("&"),
            };
            url.set_query(Some(&query_string));
            url
        };

        let result_string = self
            .http_client
            .get(search_url)
            .headers(DEFAULT_HEADERS.clone())
            .send()
            .await?
            .text()
            .await?;

        Self::check_api_error(&result_string)?;

        let result: OpenSubtitlesSubtitleResponse = serde_json::from_str(&result_string)?;

        Ok(result
            .data
            .into_iter()
            .map(move |mut open_subtitles_subtitle| {
                let files = std::mem::take(&mut open_subtitles_subtitle.attributes.files);
                let first_file = files
                    .into_iter()
                    .next()
                    .expect("There should be at least one file");

                crate::SubtitleDownloadOption {
                    id: first_file.file_id,
                    title: first_file.file_name,
                    download_count: open_subtitles_subtitle.attributes.download_count,
                    language: language.clone(),
                }
            }))
    }

    async fn download(&self, id: &Self::SubtitleId) -> core::result::Result<String, Self::Error> {
        let url = OPEN_SUBTITLES_BASE_URL
            .join("download")
            .expect("download URL has to be valid");

        let download_link_response_string = self
            .http_client
            .post(url)
            .form(&DownloadForm { file_id: *id })
            .headers(DEFAULT_HEADERS.clone())
            .send()
            .await?
            .text()
            .await?;

        Self::check_api_error(&download_link_response_string)?;

        let download_response: DownloadResponse =
            serde_json::from_str(&download_link_response_string)?;

        info!(
            "Open Subtitles: Remaining subtitle download limit: {}. Message: {}",
            download_response.remaining, download_response.message,
        );

        let download_url = reqwest::Url::from_str(&download_response.link)?;

        let subtitle_text = self
            .http_client
            .get(download_url)
            .headers(DEFAULT_HEADERS.clone())
            .send()
            .await?
            .text()
            .await?;

        Ok(subtitle_text)
    }
}

#[derive(Debug)]
pub enum Error {
    RequestError { inner: reqwest::Error },
    OpenSubtitlesAPIError { inner: OpenSubtitlesError },
    OpenSubtitlesJSONParsingError { inner: serde_json::Error },
    OpenSubtitlesInvalidURLError { inner: url::ParseError },
}

type Result<T> = core::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::RequestError { inner: value }
    }
}

impl From<OpenSubtitlesError> for Error {
    fn from(value: OpenSubtitlesError) -> Self {
        Self::OpenSubtitlesAPIError { inner: value }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::OpenSubtitlesJSONParsingError { inner: value }
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::OpenSubtitlesInvalidURLError { inner: value }
    }
}

#[cfg(test)]
mod tests {
    use domain::language::LanguageCode;

    use crate::{SubtitleProvider, open_subtitles::OpenSubtitlesClient};

    #[tokio::test]
    async fn get_movie_subtitles() {
        let client = OpenSubtitlesClient::new();

        let result: Vec<_> = client
            .search("Idiocracy", LanguageCode::Turkish, None)
            .await
            .unwrap()
            .collect();

        dbg!(&result);

        assert!(!result.is_empty())
    }

    #[tokio::test]
    async fn get_series_subtitles() {
        let client = OpenSubtitlesClient::new();

        let result: Vec<_> = client
            .search(
                "Rick and Morty",
                LanguageCode::Turkish,
                Some(domain::series::EpisodeIdentifier {
                    season_no: 1,
                    episode_no: 1,
                }),
            )
            .await
            .unwrap()
            .collect();

        dbg!(&result);

        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn search_and_download_subtitles() {
        let client = OpenSubtitlesClient::new();

        let search_result: Vec<_> = client
            .search(
                "Rick and Morty",
                LanguageCode::Turkish,
                Some(domain::series::EpisodeIdentifier {
                    season_no: 1,
                    episode_no: 1,
                }),
            )
            .await
            .unwrap()
            .collect();

        dbg!(&search_result);
        let first_result = search_result.first().unwrap();

        let download_result = client.download(&first_result.id).await.unwrap();

        dbg!(&download_result);
        assert!(!download_result.is_empty());
    }
}
