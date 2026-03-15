use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use domain::{Media, SeasonContents, language::LanguageCode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NavigationOperation {
    Push(Screen),
    ReplaceRoot(Screen),
    Reset(Option<Screen>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Startup,
    ServerAddressEntry,
    List,
    Detail(Media),
    Settings,
    Player,
    MediaManager,
    MediaManagerDetail(Media),
    // TODO make this smaller
    MediaManagerSeason {
        media: Media,
        season: u32,
        contents: SeasonContents,
        show_download_modal: bool,
    },
    ServerFileMapping(String),
    AddDownload,
    SubtitleSelection {
        media: Media,
        season: u32,
        pre_selected_episodes: Vec<u32>,
        pre_selected_language: LanguageCode,
    },
    SubtitleSearchResult {
        media_id: String,
        language: LanguageCode,
        episodes: Option<(u32, Vec<u32>)>,
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

#[must_use]
pub fn reset<Effect, Event>(
    screen: Option<Screen>,
) -> RequestBuilder<Effect, Event, impl Future<Output = ()>>
where
    Effect: Send + From<Request<NavigationOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(NavigationOperation::Reset(screen))
}
