use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use domain::Media;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NavigationOperation {
    Navigate(Screen),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Startup,
    ServerAddressEntry,
    List,
    Detail(Media),
    Settings,
    Player(String),
}

impl Operation for NavigationOperation {
    type Output = ();
}

#[must_use]
pub fn navigate<Effect, Event>(
    to: Screen,
) -> RequestBuilder<Effect, Event, impl Future<Output = ()>>
where
    Effect: Send + From<Request<NavigationOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(NavigationOperation::Navigate(to))
}
