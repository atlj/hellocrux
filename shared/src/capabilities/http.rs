use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpOperation {
    Get(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpOutput {
    Success {
        data: Option<String>,
        status_code: i32,
    },
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerConnectionState {
    Connected,
    Pending,
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
