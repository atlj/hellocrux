use std::collections::HashMap;

use domain::Media;

use crate::{
    Model, PartialModel,
    capabilities::{
        http,
        navigation::{self, Screen},
    },
    features::{query::QueryState, utils::update_model},
};

pub fn handle_get_media(model: &Model) -> crate::Command {
    let base_url = model.base_url.clone();
    let last_known_movies = model.media_items.get_data().cloned();

    crate::Command::new(|ctx| async move {
        let url = {
            let mut url = if let Some(url) = base_url {
                url
            } else {
                return navigation::push(Screen::ServerAddressEntry)
                    .into_future(ctx)
                    .await;
            };

            url.set_path("get_movies");
            url
        };

        update_model(
            &ctx,
            PartialModel {
                media_items: Some(QueryState::Loading {
                    data: last_known_movies,
                }),
                ..Default::default()
            },
        );

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
                    })
                    .unwrap_or_else(HashMap::new);

                update_model(
                    &ctx,
                    PartialModel {
                        media_items: Some(QueryState::Success { data: movies }),
                        ..Default::default()
                    },
                );
            }
            http::HttpOutput::Error => {
                // TODO: add logging
            }
        }
    })
}
