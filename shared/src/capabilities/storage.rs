use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageOperation {
    Store { key: String, value: String },
    Get(String),
}

impl Operation for StorageOperation {
    type Output = Option<String>;
}

#[must_use]
pub fn store<Effect, Event>(
    key: &str,
    value: String,
) -> RequestBuilder<Effect, Event, impl Future<Output = Option<String>>>
where
    Effect: Send + From<Request<StorageOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(StorageOperation::Store {
        key: key.to_string(),
        value,
    })
}

#[must_use]
pub fn store_with_key_string<Effect, Event>(
    key: String,
    value: String,
) -> RequestBuilder<Effect, Event, impl Future<Output = Option<String>>>
where
    Effect: Send + From<Request<StorageOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(StorageOperation::Store { key, value })
}

#[must_use]
pub fn get<Effect, Event>(
    key: &str,
) -> RequestBuilder<Effect, Event, impl Future<Output = Option<String>>>
where
    Effect: Send + From<Request<StorageOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(StorageOperation::Get(key.to_string()))
}
