use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Pending,
    Error,
    Successfull,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerCommunicationEvent {
    TryConnecting(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerCommunicationOperation {
    Connect(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerCommunicationOutput {
    ConnectionResult(bool, String),
}

impl Operation for ServerCommunicationOperation {
    type Output = ServerCommunicationOutput;
}

#[must_use]
pub fn connect<Effect, Event>(
    to: String,
) -> RequestBuilder<Effect, Event, impl Future<Output = ServerCommunicationOutput>>
where
    Effect: Send + From<Request<ServerCommunicationOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(ServerCommunicationOperation::Connect(to))
}
