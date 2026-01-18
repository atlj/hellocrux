use std::str::FromStr;

use url::Url;

const BASE_URL: &str = "https://api.opensubtitles.com/api/v1/";
const API_KEY: &str = include_str!("../../open_subtitles_api_key");

#[derive(Debug)]
pub struct SubtitleProvider<'a, F, E, R>
where
    // TODO potentially use a better type for headers.
    // But since we'll eventually use crux to pass messages, it probably has to be a sound type.
    F: Fn(Url, Vec<(String, String)>) -> R,
    R: Future<Output = Result<String, E>>,
{
    pub get: &'a F,
}

impl<F, E, R> domain::subtitles::SubtitleProvider for SubtitleProvider<'_, F, E, R>
where
    F: Fn(Url, Vec<(String, String)>) -> R,
    R: Future<Output = Result<String, E>>,
{
    type Identifier = String;
    type Error = E;

    async fn search_subtitles(
        &self,
        title: &str,
        language: domain::language::LanguageCode,
        episode: Option<domain::series::EpisodeIdentifier>,
    ) -> Result<impl Iterator<Item = Self::Identifier>, Self::Error> {
        let url = {
            let mut url = Url::from_str(BASE_URL).unwrap().join("subtitles").unwrap();

            let query_string = match episode {
                Some(domain::series::EpisodeIdentifier {
                    season_no,
                    episode_no,
                }) => format!(
                    "ai_translated=exclude&episode_number={episode_no}&languages={}&order_by=attributes%2Edownload_count&query={}&season_number={season_no}&type=episode",
                    language.to_iso639_1(),
                    percent_encoding::utf8_percent_encode(
                        title,
                        percent_encoding::NON_ALPHANUMERIC
                    )
                ),

                None => format!(
                    "ai_translated=exclude&languages={}&order_by=attributes%2Edownload_count&query={}&type=movie",
                    language.to_iso639_1(),
                    percent_encoding::utf8_percent_encode(
                        title,
                        percent_encoding::NON_ALPHANUMERIC
                    )
                ),
            };

            url.set_query(Some(&query_string));

            url
        };

        let result = (self.get)(
            url,
            vec![
                ("User-Agent".to_string(), "Streamy v0.0.1".to_string()),
                ("Api-Key".to_string(), API_KEY.trim_end().to_string()),
                ("Accept".to_string(), "application/json".to_string()),
            ],
        )
        .await?;

        Ok(std::iter::once(result))
    }

    async fn download_subtitles(
        &self,
        identifier: &Self::Identifier,
    ) -> Result<String, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use domain::{language::LanguageCode, subtitles::SubtitleProvider as _};
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    use url::Url;

    use crate::SubtitleProvider;

    #[tokio::test]
    async fn test_seach_subtitles() {
        let client = reqwest::Client::new();

        let get = |url: Url, headers: Vec<(String, String)>| async {
            let header_map = HeaderMap::from_iter(headers.into_iter().map(|(key, value)| {
                (
                    HeaderName::from_str(&key).unwrap(),
                    HeaderValue::from_str(&value).unwrap(),
                )
            }));

            let response = client
                .get(url)
                .headers(header_map)
                .send()
                .await
                .map_err(|_| ())?;
            let result = response.text().await.map_err(|_| ())?;
            Ok::<String, ()>(result)
        };

        let provider = SubtitleProvider { get: &get };

        let res = provider
            .search_subtitles("Toy Story", LanguageCode::Turkish, None)
            .await
            .unwrap()
            .next()
            .unwrap();

        dbg!(res);

        panic!()
    }
}
