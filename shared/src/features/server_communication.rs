use crux_core::{Command, render::render};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    Effect, Event, Model, PartialModel,
    capabilities::{
        http,
        navigation::{self, Screen},
        service_discovery::{self, DiscoveredService},
        storage::{self, store},
    },
    features::query::QueryState,
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
            model.connection_state = Some(QueryState::Loading {
                data: model.connection_state.as_ref().map(|_| ()),
            });

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
                            connection_state: Some(Some(QueryState::Error {
                                message: "Faulty URL passed".to_string(),
                            })),
                            ..Default::default()
                        },
                    );
                    return;
                };

                url.set_path("health");
                let connection_state = match http::get(url.clone()).into_future(ctx.clone()).await {
                    http::HttpOutput::Success { .. } => QueryState::Success { data: () },
                    http::HttpOutput::Error => QueryState::Error {
                        message: "Couldn't connect to server".to_string(),
                    },
                };

                let is_error = connection_state.is_error();

                url.set_path("");

                let base_url = if connection_state.is_success() {
                    Some(Some(url.clone()))
                } else {
                    None
                };

                update_model(
                    &ctx,
                    PartialModel {
                        connection_state: Some(Some(connection_state)),
                        base_url,
                        ..Default::default()
                    },
                );

                if is_error {
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
