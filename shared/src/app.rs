use crux_core::{
    App, Command,
    macros::effect,
    render::{RenderOperation, render},
};
use partially::Partial;
use serde::{Deserialize, Serialize};

use crate::capabilities::{
    navigation::{NavigationOperation, Screen, navigate},
    server_communication::{
        ConnectionState, ServerCommunicationEvent, ServerCommunicationOperation,
        ServerCommunicationOutput, connect,
    },
    storage::{StorageOperation, get},
};

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
    ServerCommunication(ServerCommunicationOperation),
}

#[derive(Default, Partial, Clone, Debug, Serialize, Deserialize)]
#[partially(derive(Debug, Clone, Serialize, Deserialize))]
pub struct Model {
    current_screen: Screen,
    server_address: Option<String>,
    connection_state: Option<ConnectionState>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ViewModel {
    current_screen: Screen,
    connection_state: Option<ConnectionState>,
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
                get("server_address")
                    .then_request(|stored_server_address| match stored_server_address {
                        None => navigate::<Effect, Event>(Screen::ServerAddressEntry),
                        Some(_) => navigate(Screen::List),
                    })
                    .into_future(ctx)
                    .await;
            }),
            Event::ServerCommunication(event) => match event {
                ServerCommunicationEvent::TryConnecting(address) => {
                    model.connection_state = Some(ConnectionState::Pending);
                    connect::<Effect, Event>(address).then_send(|output| -> Event {
                        match output {
                            ServerCommunicationOutput::ConnectionResult(result) => {
                                Event::UpdateModel(PartialModel {
                                    current_screen: None,
                                    server_address: None,
                                    connection_state: Some(Some(if result {
                                        ConnectionState::Successfull
                                    } else {
                                        ConnectionState::Error
                                    })),
                                })
                            }
                        }
                    })
                }
            },
            Event::UpdateModel(partial_model) => {
                model.apply_some(partial_model);
                render()
            }
            Event::ScreenChanged(screen) => todo!(),
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            current_screen: model.current_screen.clone(),
            connection_state: model.connection_state.clone(),
        }
    }
}
