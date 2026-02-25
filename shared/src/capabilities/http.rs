use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpOperation {
    Get(String),
    Post { url: String, body: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpOutput {
    Success {
        data: Option<String>,
        status_code: i32,
    },
    Error,
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
    Command::request_from_shell(HttpOperation::Get(url.to_string()))
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
