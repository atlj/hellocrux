use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use domain::Media;
use serde::{Deserialize, Serialize};

use crate::features::playback::Episode;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NavigationOperation {
    Push(Screen),
    ReplaceRoot(Screen),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Startup,
    ServerAddressEntry,
    List,
    Detail(Media),
    Settings,
    Player {
        id: String,
        url: String,
        episode: Option<Episode>,
        initial_seconds: Option<u64>,
    },
}

impl Operation for NavigationOperation {
    type Output = ();
}

#[must_use]
pub fn replace_root<Effect, Event>(
    to: Screen,
) -> RequestBuilder<Effect, Event, impl Future<Output = ()>>
where
    Effect: Send + From<Request<NavigationOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(NavigationOperation::ReplaceRoot(to))
}

#[must_use]
pub fn push<Effect, Event>(to: Screen) -> RequestBuilder<Effect, Event, impl Future<Output = ()>>
where
    Effect: Send + From<Request<NavigationOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(NavigationOperation::Push(to))
}
