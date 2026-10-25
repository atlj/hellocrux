use crate::capabilities::service_discovery::{DiscoveredService, ServiceDiscoveryOperation};
use crate::features;
use crate::features::data::DataRequest;
use crate::features::playback::PlaybackModel;
use crate::features::query::QueryState;
use crate::features::query::view_model_queries::{
    ActionState, MediaItems, MediaItemsContent, SubtitleSearchResults, SubtitleSearchState,
    TMDBResults,
};
use crate::features::subtitle::SubtitleEvent;
use crate::features::tmdb::TMDBModel;
use crate::features::{
    playback::{PlayEvent, PlaybackPosition},
    server_communication::ServerCommunicationEvent,
};
use crux_core::command::CommandContext;
use crux_core::{App, macros::effect, render::RenderOperation};
use domain::series::SeriesFileMapping;
use domain::{Download, PreparingMedia};
use partially::Partial;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::capabilities::{
    http::HttpOperation,
    navigation::{NavigationOperation, Screen},
    storage::StorageOperation,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Startup,
    ScreenChanged(Screen),
    UpdateData(DataRequest),
    ServerCommunication(ServerCommunicationEvent),
    Play(PlayEvent),
    PlaybackProgress((u64, PlaybackPosition)),
    Subtitle(SubtitleEvent),

    #[serde(skip)]
    UpdateModel(Box<PartialModel>),
    #[serde(skip)]
    PushIfNecessary(Screen),
}

#[effect(typegen)]
pub enum Effect {
    Render(RenderOperation),
    Store(StorageOperation),
    Navigate(NavigationOperation),
    Http(HttpOperation),
    ServiceDiscovery(ServiceDiscoveryOperation),
}

pub type Command = crux_core::Command<Effect, Event>;

pub type CruxContext = CommandContext<Effect, Event>;

#[derive(Default, Partial, Clone, Debug)]
#[partially(derive(Debug, Clone, Default))]
pub struct Model {
    pub base_url: Option<Url>,
    pub current_screen: Screen,
    pub connection_state: Option<QueryState<()>>,
    pub media_items: QueryState<MediaItemsContent>,
    pub downloads: Vec<Download>,
    pub preparing_list: Vec<PreparingMedia>,
    pub torrent_contents: Option<(String, SeriesFileMapping)>,
    pub playback: PlaybackModel,
    pub discovered_services: Vec<DiscoveredService>,
    pub last_played_media_id: Option<String>,

    pub tmdb_search_results: Option<(String, QueryState<tmdb::PaginatedMultiSearchResult>)>,

    // TODO consolidate
    pub subtitles_search_results: QueryState<SubtitleSearchResults>,
    pub subtitle_download_results: Option<QueryState<()>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ViewModel {
    connection_state: Option<ActionState>,
    media_items: MediaItems,
    downloads: Vec<Download>,
    preparing_list: Vec<PreparingMedia>,
    playback_detail: PlaybackModel,
    torrent_contents: Option<(String, SeriesFileMapping)>,
    discovered_services: Vec<DiscoveredService>,
    last_played_media_id: Option<String>,
    tmdb_search_results: Option<(String, TMDBResults)>,

    // TODO consolidate
    subtitle_search_results: SubtitleSearchState,
    subtitle_download_results: Option<ActionState>,
}

#[derive(Default)]
pub struct CounterApp;

impl App for CounterApp {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;
    type Capabilities = ();

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
        _caps: &Self::Capabilities,
    ) -> Command {
        match event {
            Event::Startup => features::lifetime::handle_startup(model),
            Event::ScreenChanged(screen) => features::lifetime::handle_screen_change(model, screen),
            Event::UpdateData(request) => features::data::update_data(model, request),
            Event::ServerCommunication(event) => {
                features::server_communication::handle_server_communication(model, event)
            }
            Event::Play(play_event) => features::playback::handle_play(model, play_event),
            Event::PlaybackProgress((duration_seconds, playback_progress_data)) => {
                features::playback::handle_playback_progress(
                    model,
                    duration_seconds,
                    playback_progress_data,
                )
            }
            Event::UpdateModel(partial_model) => {
                features::utils::handle_update_model(model, partial_model)
            }
            Event::PushIfNecessary(screen) => {
                features::utils::handle_push_if_necessary(model, screen)
            }
            Event::Subtitle(subtitle_event) => {
                features::subtitle::handle_subtitle_event(model, subtitle_event)
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            connection_state: model.connection_state.clone().map(ActionState::from),
            media_items: model.media_items.clone().into(),
            playback_detail: model.playback.clone(),
            downloads: model.downloads.clone(),
            preparing_list: model.preparing_list.clone(),
            torrent_contents: model.torrent_contents.clone(),
            discovered_services: model.discovered_services.clone(),
            last_played_media_id: model.last_played_media_id.clone(),
            tmdb_search_results: model
                .tmdb_search_results
                .clone()
                .map(|(query, state)| (query, state.into())),
            subtitle_search_results: model.subtitles_search_results.clone().into(),
            subtitle_download_results: model
                .subtitle_download_results
                .clone()
                .map(ActionState::from),
        }
    }
}
