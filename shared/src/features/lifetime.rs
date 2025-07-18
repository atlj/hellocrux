use crux_core::{Command, render::render};
use domain::Media;
use url::Url;

use crate::{
    Effect, Event, Model, PartialModel,
    capabilities::{
        http,
        navigation::{Screen, navigate},
        storage,
    },
};

pub fn handle_startup(_: &mut Model) -> Command<Effect, Event> {
    Command::new(|ctx| async move {
        let stored_server_address = storage::get("server_address")
            .into_future(ctx.clone())
            .await;
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
    })
}

pub fn handle_screen_change(model: &mut Model, screen: Screen) -> Command<Effect, Event> {
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
                        let movies =
                            data.and_then(|data| serde_json::from_str::<Vec<Media>>(&data).ok());

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
        Screen::Detail(_) => Command::done(),
        Screen::Settings => Command::done(),
        Screen::Player { .. } => Command::done(),
    }
}
