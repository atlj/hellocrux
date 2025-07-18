use crux_core::{Command, render::render};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    Effect, Event, Model, PartialModel,
    capabilities::{
        http::{self, HttpRequestState},
        navigation::{Screen, navigate},
        storage::store,
    },
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerCommunicationEvent {
    TryConnecting(String),
}

pub fn handle_server_communication(
    model: &mut Model,
    event: ServerCommunicationEvent,
) -> Command<Effect, Event> {
    match event {
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
                let connection_state = match http::get(url.clone()).into_future(ctx.clone()).await {
                    http::HttpOutput::Success { .. } => HttpRequestState::Success,
                    http::HttpOutput::Error => HttpRequestState::Error,
                };

                url.set_path("");

                ctx.send_event(Event::UpdateModel(PartialModel {
                    connection_state: Some(Some(connection_state.clone())),
                    base_url: if matches!(connection_state, HttpRequestState::Success) {
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
    }
}
