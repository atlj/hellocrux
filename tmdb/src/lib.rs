use std::{fmt::Display, sync::LazyLock};

use url::Url;

pub use dto::*;
mod dto;

const API_KEY: &str = concat!("Bearer ", include_str!("../../tmdb_api_key"));
const BASE_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.themoviedb.org/3/").unwrap());
const MOVIE_BASE_URL: &str = "https://image.tmdb.org/t/p/original/";

const HEADERS: LazyLock<[(&str, &str); 2]> = LazyLock::new(|| {
    [
        ("Accept", "application/json"),
        ("Authorization", API_KEY.trim_end()),
    ]
});

pub async fn multi_search<Context, APICall, Error>(
    api_call: APICall,
    context: Context,
    query: &str,
    page: usize,
) -> Result<PaginatedMultiSearchResult>
where
    Context: std::any::Any,
    APICall: AsyncFnOnce(Context, Url, &[(&str, &str)]) -> core::result::Result<String, Error>,
    Error: Display,
{
    let query =
        serde_urlencoded::to_string([("query", query), ("page", page.to_string().as_ref())])
            .unwrap();
    let url = {
        let mut url = BASE_URL.join("search/multi").unwrap();
        url.set_query(Some(&query));
        url
    };

    let api_result = api_call(context, url, HEADERS.as_ref())
        .await
        .map_err(|err| crate::Error::FailedAPICall {
            reason: err.to_string(),
        })?;

    let PaginatedMultiSearchResultNonJoined {
        page,
        total_pages,
        total_results,
        results,
    } = match serde_json::from_str(&api_result) {
        Ok(result) => result,
        Err(err) => {
            let Ok(api_error) = serde_json::from_str::<dto::ErrorMessage>(&api_result) else {
                return Err(crate::Error::JSONParse {
                    source: err,
                    original_str: api_result.to_string(),
                });
            };

            if api_error.status_message.starts_with("Invalid API Key") {
                return Err(crate::Error::InvalidAPIKey);
            }

            return Err(crate::Error::APIError(api_error.status_message));
        }
    };

    Ok(PaginatedMultiSearchResult {
        page,
        total_pages,
        total_results,
        results: results
            .into_iter()
            .filter_map(|result| result.try_into().ok())
            .map(|result: MultiSearchResult| result.with_prepended_poster_path())
            .collect(),
    })
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API call was not successful, reason: {reason}")]
    FailedAPICall { reason: String },
    #[error(
        "Couldn't parse API response as JSON, reason: {source}. Original string: {original_str}"
    )]
    JSONParse {
        #[source]
        source: serde_json::Error,
        original_str: String,
    },
    #[error("API key is invalid")]
    InvalidAPIKey,
    #[error("API returned an error: {0}")]
    APIError(String),
}

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderName, HeaderValue};
    use url::Url;

    async fn api_call(_: (), url: Url, headers: &[(&str, &str)]) -> Result<String, reqwest::Error> {
        dbg!(url.to_string());

        let header_map = {
            let mut header_map = reqwest::header::HeaderMap::new();

            for (key, value) in headers {
                dbg!((key, value));
                let key: HeaderName = key.parse().unwrap();
                let value: HeaderValue = value.parse().unwrap();
                header_map.insert(key, value);
            }

            header_map
        };

        let response = reqwest::Client::default()
            .get(url)
            .headers(header_map)
            .send()
            .await?;

        let text = response.text().await?;
        dbg!(&text);

        Ok(text)
    }

    #[tokio::test]
    #[ignore = "Test is depending on external API"]
    async fn test_basic_search() {
        let result = crate::multi_search(api_call, (), "forrest gump", 1)
            .await
            .unwrap();

        assert!(!result.results.is_empty());

        // Make sure we are discarding people
        let result = crate::multi_search(api_call, (), "fo", 1).await.unwrap();

        assert!(!result.results.is_empty());
    }
}
