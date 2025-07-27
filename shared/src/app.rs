use std::collections::HashMap;

use crate::features;
use crate::features::playback::PlaybackModel;
use crate::features::{
    playback::{PlayEvent, PlaybackPosition},
    server_communication::ServerCommunicationEvent,
};
use crux_core::command::CommandContext;
use crux_core::{App, Command, macros::effect, render::RenderOperation};
use domain::Media;
use partially::Partial;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::capabilities::{
    http::{HttpOperation, ServerConnectionState},
    navigation::{NavigationOperation, Screen},
    storage::StorageOperation,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Startup,
    ScreenChanged(Screen),
    ServerCommunication(ServerCommunicationEvent),
    Play(PlayEvent),
    PlaybackProgress((u64, PlaybackPosition)),

    #[serde(skip)]
    UpdateModel(Box<PartialModel>),
}

#[effect(typegen)]
pub enum Effect {
    Render(RenderOperation),
    Store(StorageOperation),
    Navigate(NavigationOperation),
    Http(HttpOperation),
}

pub type CruxContext = CommandContext<Effect, Event>;

#[derive(Default, Partial, Clone, Debug)]
#[partially(derive(Debug, Clone, Default))]
pub struct Model {
    pub base_url: Option<Url>,
    pub current_screen: Screen,
    pub connection_state: Option<ServerConnectionState>,
    pub media_items: Option<HashMap<String, Media>>,
    pub playback: PlaybackModel,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ViewModel {
    connection_state: Option<ServerConnectionState>,
    media_items: Option<HashMap<String, Media>>,
    playback_detail: PlaybackModel,
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
    ) -> Command<Self::Effect, Self::Event> {
        match event {
            // Lifetime
            Event::Startup => features::lifetime::handle_startup(model),
            Event::ScreenChanged(screen) => features::lifetime::handle_screen_change(model, screen),

            // Server communication
            Event::ServerCommunication(event) => {
                features::server_communication::handle_server_communication(model, event)
            }

            // Playback
            Event::Play(play_event) => features::playback::handle_play(model, play_event),
            Event::PlaybackProgress((duration_seconds, playback_progress_data)) => {
                features::playback::handle_playback_progress(
                    model,
                    duration_seconds,
                    playback_progress_data,
                )
            }

            // Utils
            Event::UpdateModel(partial_model) => {
                features::utils::handle_update_model(model, partial_model)
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            connection_state: model.connection_state.clone(),
            media_items: model.media_items.clone(),
            playback_detail: model.playback.clone(),
        }
    }
}
