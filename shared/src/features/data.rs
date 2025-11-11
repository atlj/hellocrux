use std::collections::HashMap;

use crux_core::Command;
use domain::Media;

use crate::{
    Effect, Event, Model, PartialModel,
    capabilities::{
        http,
        navigation::{self, Screen},
    },
};

use super::utils::update_model;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum DataRequest {
    Media,
    Downloads,
}

pub fn update_data(model: &mut Model, request: DataRequest) -> Command<Effect, Event> {
    let base_url = model.base_url.clone();

    match request {
        DataRequest::Media => Command::new(|ctx| async move {
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
        DataRequest::Downloads => todo!(),
    }
}
