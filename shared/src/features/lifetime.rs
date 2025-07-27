use std::collections::HashMap;

use crux_core::{Command, render::render};
use domain::Media;
use url::Url;

use crate::{
    Effect, Event, Model, PartialModel,
    capabilities::{
        http,
        navigation::{self, Screen},
        storage,
    },
    features::utils::update_model,
};

use super::playback::{PlayEvent, PlaybackModel, PlaybackPosition};

pub fn handle_startup(_: &mut Model) -> Command<Effect, Event> {
    Command::new(|ctx| async move {
        let server_addres = if let Some(address) = storage::get("server_address")
            .into_future(ctx.clone())
            .await
        {
            address
        } else {
            return navigation::replace_root(Screen::ServerAddressEntry)
                .into_future(ctx)
                .await;
        };

        update_model(
            &ctx,
            PartialModel {
                base_url: Some(Some(Url::parse(&server_addres).unwrap())),
                ..Default::default()
            },
        );

        navigation::replace_root(Screen::List)
            .into_future(ctx)
            .await;
    })
}

pub fn handle_screen_change(model: &mut Model, screen: Screen) -> Command<Effect, Event> {
    model.current_screen = screen.clone();
    let base_url = model.base_url.clone();

    let command = match screen {
        Screen::List => Command::new(|ctx| async move {
            let mut url = if let Some(url) = base_url {
                url
            } else {
                return navigation::push(Screen::ServerAddressEntry)
                    .into_future(ctx)
                    .await;
            };

            url.set_path("get_movies");

            match http::get(url).into_future(ctx.clone()).await {
                http::HttpOutput::Success { data, .. } => {
                    let movies = data
                        .and_then(|data| serde_json::from_str::<Vec<Media>>(&data).ok())
                        .map(|movies| {
                            let mut movies_hashmap = HashMap::with_capacity(movies.len());

                            movies.into_iter().for_each(|media| {
                                movies_hashmap.insert(media.id.clone(), media);
                            });

                            movies_hashmap
                        });

                    update_model(
                        &ctx,
                        PartialModel {
                            media_items: Some(movies),
                            ..Default::default()
                        },
                    );
                }
                http::HttpOutput::Error => {
                    // log
                }
            }
        }),
        Screen::Detail(Media { id, .. }) => Command::new(|ctx| async move {
            let (initial_seconds, episode) = PlayEvent::FromSavedPosition { id: id.clone() }
                .get_position(ctx.clone())
                .await;

            let position = initial_seconds.map(|position_seconds| match episode {
                None => PlaybackPosition::Movie {
                    id,
                    position_seconds,
                },
                Some(episode_identifier) => PlaybackPosition::SeriesEpisode {
                    id,
                    episode_identifier,
                    position_seconds,
                },
            });

            update_model(
                &ctx,
                PartialModel {
                    playback: Some(PlaybackModel {
                        last_position: position,
                        active_player: None,
                    }),
                    ..Default::default()
                },
            );
        }),

        Screen::Startup => Command::done(),
        Screen::ServerAddressEntry => Command::done(),
        Screen::Settings => Command::done(),
        Screen::Player => Command::done(),
    };

    render().and(command)
}
