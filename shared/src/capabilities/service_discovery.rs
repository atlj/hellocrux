use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceDiscoveryOperation {
    Start,
    Stop,
}

impl Operation for ServiceDiscoveryOperation {
    type Output = ();
}

#[must_use]
pub fn start<Effect, Event>() -> RequestBuilder<Effect, Event, impl Future<Output = ()>>
where
    Effect: Send + From<Request<ServiceDiscoveryOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(ServiceDiscoveryOperation::Start)
}

#[must_use]
pub fn stop<Effect, Event>() -> RequestBuilder<Effect, Event, impl Future<Output = ()>>
where
    Effect: Send + From<Request<ServiceDiscoveryOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(ServiceDiscoveryOperation::Stop)
}
