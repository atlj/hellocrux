mod dto;
use std::future::Future;
use std::str::FromStr;

use url::Url;

const BASE_URL: &str = "https://api.opensubtitles.com/api/v1/";
const API_KEY: &str = include_str!("../../open_subtitles_api_key");

pub struct SubtitleProvider<C: HttpClient> {
    client: C,
}

pub trait HttpClient {
    type Error;

    // TODO potentially use a better type for headers.
    // But since we'll eventually use crux to pass messages, it probably has to be a sound type.
    fn get(
        &self,
        url: Url,
        headers: Vec<(String, String)>,
    ) -> impl Future<Output = Result<String, Self::Error>>;

    fn post<Form: serde::Serialize>(
        &self,
        url: Url,
        form: &Form,
        headers: Vec<(String, String)>,
    ) -> impl Future<Output = Result<String, Self::Error>>;
}

#[derive(Debug)]
pub struct Subtitle {
    pub title: String,
    pub id: usize,
    pub download_count: usize,
}

impl std::fmt::Display for Subtitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<C: HttpClient> SubtitleProvider<C> {
    fn default_headers() -> Vec<(String, String)> {
        vec![
            ("User-Agent".to_string(), "Streamy v0.0.1".to_string()),
            ("Api-Key".to_string(), API_KEY.trim_end().to_string()),
            ("Accept".to_string(), "application/json".to_string()),
        ]
    }
}

impl<C: HttpClient> domain::subtitles::SubtitleProvider for SubtitleProvider<C> {
    type Identifier = Subtitle;
    type Error = Error<C::Error>;

    async fn search_subtitles(
        &self,
        title: &str,
        language: domain::language::LanguageCode,
        episode: Option<domain::series::EpisodeIdentifier>,
    ) -> Result<impl Iterator<Item = Self::Identifier>, Self::Error> {
        let mut url = Url::from_str(BASE_URL).unwrap().join("subtitles").unwrap();

        let encoded_title =
            percent_encoding::utf8_percent_encode(title, percent_encoding::NON_ALPHANUMERIC);

        let query_string = match episode {
            Some(domain::series::EpisodeIdentifier {
                season_no,
                episode_no,
            }) => format!(
                "ai_translated=exclude&episode_number={episode_no}&languages={}&order_by=attributes%2Edownload_count&query={encoded_title}&season_number={season_no}&type=episode",
                language.to_iso639_1(),
            ),
            None => format!(
                "ai_translated=exclude&languages={}&order_by=attributes%2Edownload_count&query={encoded_title}&type=movie",
                language.to_iso639_1(),
            ),
        };
        url.set_query(Some(&query_string));

        let result = self
            .client
            .get(url, Self::default_headers())
            .await
            .map_err(Error::Request)?;

        let parsed: dto::OpenSubtitlesSubtitleResponse = serde_json::from_str(&result)?;
        Ok(parsed
            .data
            .into_iter()
            .map(dto::OpenSubtitlesSubtitle::into))
    }

    async fn download_subtitles(
        &self,
        identifier: &Self::Identifier,
    ) -> Result<String, Self::Error> {
        let url = Url::from_str(BASE_URL).unwrap().join("download").unwrap();

        let response = self
            .client
            .post(
                url,
                &dto::DownloadForm {
                    file_id: identifier.id,
                },
                Self::default_headers(),
            )
            .await
            .map_err(Error::Request)?;

        let parsed: dto::DownloadResponse = serde_json::from_str(&response)?;

        let subtitle_contents = self
            .client
            .get(Url::from_str(&parsed.link).unwrap(), vec![])
            .await
            .map_err(Error::Request)?;

        Ok(subtitle_contents)
    }
}

#[derive(Debug)]
pub enum Error<Inner> {
    Request(Inner),
    Deserialize(serde_json::Error),
}

impl<Inner: std::fmt::Display + std::fmt::Debug> std::fmt::Display for Error<Inner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<Inner> From<serde_json::Error> for Error<Inner> {
    fn from(value: serde_json::Error) -> Self {
        Self::Deserialize(value)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use domain::{
        language::LanguageCode, series::EpisodeIdentifier, subtitles::SubtitleProvider as _,
    };
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    use url::Url;

    use crate::{HttpClient, Subtitle, SubtitleProvider};

    struct Client {
        client: reqwest::Client,
    }

    impl HttpClient for Client {
        type Error = ();

        async fn get(
            &self,
            url: Url,
            headers: Vec<(String, String)>,
        ) -> Result<String, Self::Error> {
            let header_map = HeaderMap::from_iter(headers.into_iter().map(|(key, value)| {
                (
                    HeaderName::from_str(&key).unwrap(),
                    HeaderValue::from_str(&value).unwrap(),
                )
            }));

            let response = self
                .client
                .get(url)
                .headers(header_map)
                .send()
                .await
                .map_err(|_| ())?;
            response.text().await.map_err(|_| ())
        }

        async fn post<Form: serde::Serialize>(
            &self,
            url: Url,
            form: &Form,
            headers: Vec<(String, String)>,
        ) -> Result<String, Self::Error> {
            let header_map = HeaderMap::from_iter(headers.into_iter().map(|(key, value)| {
                (
                    HeaderName::from_str(&key).unwrap(),
                    HeaderValue::from_str(&value).unwrap(),
                )
            }));

            let response = self
                .client
                .post(url)
                .headers(header_map)
                .form(form)
                .send()
                .await
                .map_err(|_| ())?;
            response.text().await.map_err(|_| ())
        }
    }

    #[tokio::test]
    #[ignore = "reason"]
    async fn test_search_subtitles() {
        let provider = SubtitleProvider {
            client: Client {
                client: reqwest::Client::new(),
            },
        };

        let res = provider
            .search_subtitles(
                "Attack on Titan",
                LanguageCode::English,
                Some(EpisodeIdentifier {
                    season_no: 2,
                    episode_no: 8,
                }),
            )
            .await
            .unwrap();

        dbg!(res.collect::<Vec<_>>());
    }

    #[tokio::test]
    #[ignore = "reason"]
    async fn test_download_subtitles() {
        let provider = SubtitleProvider {
            client: Client {
                client: reqwest::Client::new(),
            },
        };

        let res = provider
            .download_subtitles(&Subtitle {
                title: "a".to_string(),
                id: 3999587,
                download_count: 0,
            })
            .await
            .unwrap();

        dbg!(&res);
    }
}
