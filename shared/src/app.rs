use crux_core::{
    App, Command,
    macros::effect,
    render::{RenderOperation, render},
};
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

    #[serde(skip_serializing)]
    UpdateModel(PartialModel),
}

#[effect(typegen)]
pub enum Effect {
    Render(RenderOperation),
    Store(StorageOperation),
    Navigate(NavigationOperation),
    Http(HttpOperation),
}

#[derive(Default, Partial, Clone, Debug, Serialize, Deserialize)]
#[partially(derive(Debug, Clone, Serialize, Deserialize, Default))]
pub struct Model {
    current_screen: Screen,
    server_address: Option<String>,
    connection_state: Option<HttpRequestState>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ViewModel {
    current_screen: Screen,
    connection_state: Option<HttpRequestState>,
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
                    None => navigate::<Effect, Event>(Screen::ServerAddressEntry),
                    Some(_) => navigate(Screen::List),
                }
                .into_future(ctx)
                .await
            }),
            Event::UpdateModel(partial_model) => {
                model.apply_some(partial_model);
                render()
            }
            Event::ScreenChanged(screen) => {
                model.current_screen = screen;
                render()
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

                        ctx.send_event(Event::UpdateModel(PartialModel {
                            connection_state: Some(Some(connection_state.clone())),
                            ..Default::default()
                        }));

                        if matches!(connection_state, HttpRequestState::Error) {
                            return;
                        }

                        url.set_path("");
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
            current_screen: model.current_screen.clone(),
            connection_state: model.connection_state.clone(),
        }
    }
}
