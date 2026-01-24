use crux_core::{Command, render::render};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    Effect, Event, Model, PartialModel,
    capabilities::{
        http::{self, ServerConnectionState},
        navigation::{self, Screen},
        service_discovery::{self, DiscoveredService},
        storage::{self, store},
    },
};

use super::utils::update_model;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerCommunicationEvent {
    Discovered(Vec<DiscoveredService>),
    TryConnecting(String),
    Reset,
}

pub fn handle_server_communication(
    model: &mut Model,
    event: ServerCommunicationEvent,
) -> Command<Effect, Event> {
    match event {
        ServerCommunicationEvent::TryConnecting(mut address) => {
            model.connection_state = Some(ServerConnectionState::Pending);

            let command = Command::new(|ctx| async move {
                if !address.starts_with("http") {
                    address = "http://".to_owned() + &address;
                }

                let mut url = if let Ok(url) = Url::parse(&address) {
                    url
                } else {
                    update_model(
                        &ctx,
                        PartialModel {
                            connection_state: Some(Some(ServerConnectionState::Error)),
                            ..Default::default()
                        },
                    );
                    return;
                };

                url.set_path("health");
                let connection_state = match http::get(url.clone()).into_future(ctx.clone()).await {
                    http::HttpOutput::Success { .. } => ServerConnectionState::Connected,
                    http::HttpOutput::Error => ServerConnectionState::Error,
                };

                url.set_path("");

                update_model(
                    &ctx,
                    PartialModel {
                        connection_state: Some(Some(connection_state.clone())),
                        base_url: if matches!(connection_state, ServerConnectionState::Connected) {
                            Some(Some(url.clone()))
                        } else {
                            None
                        },
                        ..Default::default()
                    },
                );

                if matches!(connection_state, ServerConnectionState::Error) {
                    return;
                }

                store("server_address", url.to_string())
                    .into_future(ctx.clone())
                    .await;
                service_discovery::stop().into_future(ctx.clone()).await;
                navigation::replace_root(Screen::List)
                    .into_future(ctx)
                    .await;
            });

            render().and(command)
        }
        ServerCommunicationEvent::Reset => Command::new(|ctx| async move {
            storage::remove("server_address")
                .into_future(ctx.clone())
                .await;
            navigation::reset(Some(Screen::ServerAddressEntry))
                .into_future(ctx)
                .await;
        }),
        ServerCommunicationEvent::Discovered(services) => Command::new(|ctx| async move {
            update_model(
                &ctx,
                PartialModel {
                    discovered_services: Some(services),
                    ..Default::default()
                },
            );
        }),
    }
}
