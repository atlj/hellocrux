use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use serde::{Deserialize, Serialize};
use url::Url;

/// crux type gen converts (String, String) to [String]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderKeyValuePair {
    pub key: String,
    pub value: String,
}

impl From<(String, String)> for HeaderKeyValuePair {
    fn from((key, value): (String, String)) -> Self {
        HeaderKeyValuePair { key, value }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpOperation {
    Get {
        url: String,
        headers: Option<Vec<HeaderKeyValuePair>>,
    },
    Post {
        url: String,
        body: String,
    },
}

// TODO convert to result
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpOutput {
    Success {
        data: Option<String>,
        status_code: i32,
    },
    Error,
}

impl HttpOutput {
    pub fn into_option(self) -> Option<String> {
        match self {
            HttpOutput::Success {
                data: Some(some_data),
                ..
            } => Some(some_data),
            _ => None,
        }
    }
}

impl Operation for HttpOperation {
    type Output = HttpOutput;
}

#[must_use]
pub fn get<Effect, Event>(
    url: Url,
) -> RequestBuilder<Effect, Event, impl Future<Output = HttpOutput>>
where
    Effect: Send + From<Request<HttpOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(HttpOperation::Get {
        url: url.to_string(),
        headers: None,
    })
}

#[must_use]
pub fn get_with_headers<Effect, Event>(
    url: Url,
    headers: Vec<HeaderKeyValuePair>,
) -> RequestBuilder<Effect, Event, impl Future<Output = HttpOutput>>
where
    Effect: Send + From<Request<HttpOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(HttpOperation::Get {
        url: url.to_string(),
        headers: Some(headers),
    })
}

#[must_use]
pub fn post<Effect, Event>(
    url: Url,
    body: String,
) -> RequestBuilder<Effect, Event, impl Future<Output = HttpOutput>>
where
    Effect: Send + From<Request<HttpOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(HttpOperation::Post {
        url: url.to_string(),
        body,
    })
}
