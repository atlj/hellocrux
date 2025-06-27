use crux_core::{
    App, Command,
    macros::effect,
    render::{RenderOperation, render},
};
use domain::Media;
use partially::Partial;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::capabilities::{
    http::{self, HttpOperation, HttpRequestState},
    navigation::{NavigationOperation, Screen, navigate},
    storage::{StorageOperation, get, store},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerCommunicationEvent {
    TryConnecting(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Startup,
    ScreenChanged(Screen),
    ServerCommunication(ServerCommunicationEvent),

    #[serde(skip)]
    UpdateModel(PartialModel),
}

#[effect(typegen)]
pub enum Effect {
    Render(RenderOperation),
    Store(StorageOperation),
    Navigate(NavigationOperation),
    Http(HttpOperation),
}

#[derive(Default, Partial, Clone, Debug)]
#[partially(derive(Debug, Clone, Default))]
pub struct Model {
    base_url: Option<Url>,
    current_screen: Screen,
    connection_state: Option<HttpRequestState>,
    media_items: Option<Vec<Media>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ViewModel {
    connection_state: Option<HttpRequestState>,
    media_items: Option<Vec<Media>>,
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
            Event::Startup => Command::new(|ctx| async move {
                let stored_server_address = get("server_address").into_future(ctx.clone()).await;
                match stored_server_address {
                    None => {
                        navigate::<Effect, Event>(Screen::ServerAddressEntry)
                            .into_future(ctx)
                            .await;
                    }
                    Some(stored_address) => {
                        ctx.send_event(Event::UpdateModel(PartialModel {
                            base_url: Some(Some(Url::parse(&stored_address).unwrap())),
                            ..Default::default()
                        }));
                        navigate(Screen::List).into_future(ctx).await;
                    }
                }
            }),
            Event::UpdateModel(partial_model) => {
                model.apply_some(partial_model);
                render()
            }
            Event::ScreenChanged(screen) => {
                model.current_screen = screen.clone();
                _ = render::<Effect, Event>();

                match screen {
                    Screen::Startup => Command::done(),
                    Screen::ServerAddressEntry => Command::done(),
                    Screen::List => {
                        let mut url = if let Some(url) = model.base_url.clone() {
                            url
                        } else {
                            return Command::new(|ctx| async move {
                                navigate(Screen::ServerAddressEntry).into_future(ctx).await;
                            });
                        };

                        Command::new(|ctx| async move {
                            url.set_path("get_movies");

                            match http::get(url).into_future(ctx.clone()).await {
                                http::HttpOutput::Success { data, .. } => {
                                    let movies = data
                                        .map(|data| serde_json::from_str::<Vec<Media>>(&data).ok())
                                        .flatten();

                                    ctx.send_event(Event::UpdateModel(PartialModel {
                                        media_items: Some(movies),
                                        ..Default::default()
                                    }));
                                }
                                http::HttpOutput::Error => {
                                    // log
                                }
                            }
                        })
                    }
                    Screen::Detail(media) => Command::done(),
                    Screen::Settings => Command::done(),
                }
            }
            Event::ServerCommunication(event) => match event {
                ServerCommunicationEvent::TryConnecting(mut address) => {
                    model.connection_state = Some(HttpRequestState::Pending);
                    _ = render::<Effect, Event>();

                    Command::new(|ctx| async move {
                        if !address.starts_with("http") {
                            address = "http://".to_owned() + &address;
                        }

                        let mut url = if let Ok(url) = Url::parse(&address) {
                            url
                        } else {
                            ctx.send_event(Event::UpdateModel(PartialModel {
                                connection_state: Some(Some(HttpRequestState::Error)),
                                ..Default::default()
                            }));
                            return;
                        };

                        url.set_path("health");
                        let connection_state =
                            match http::get(url.clone()).into_future(ctx.clone()).await {
                                http::HttpOutput::Success { .. } => HttpRequestState::Success,
                                http::HttpOutput::Error => HttpRequestState::Error,
                            };

                        url.set_path("");

                        ctx.send_event(Event::UpdateModel(PartialModel {
                            connection_state: Some(Some(connection_state.clone())),
                            base_url: if matches!(
                                connection_state,
                                HttpRequestState::Success { .. }
                            ) {
                                Some(Some(url.clone()))
                            } else {
                                None
                            },
                            ..Default::default()
                        }));

                        if matches!(connection_state, HttpRequestState::Error) {
                            return;
                        }

                        store("server_address", url.to_string())
                            .into_future(ctx.clone())
                            .await;
                        navigate(Screen::List).into_future(ctx).await;
                    })
                }
            },
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            connection_state: model.connection_state.clone(),
            media_items: model.media_items.clone(),
        }
    }
}
