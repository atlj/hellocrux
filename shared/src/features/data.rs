use std::collections::HashMap;

use crux_core::Command;
use domain::{Download, DownloadForm, Media};

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
    GetMedia,
    GetDownloads,
    AddDownload(DownloadForm),
    GetContents(String),
}

pub fn update_data(model: &mut Model, request: DataRequest) -> Command<Effect, Event> {
    let base_url = model.base_url.clone();

    match request {
        DataRequest::GetContents(id) => Command::new(|ctx| async move {
            // TODO make this url common between enum vals
            let url = {
                let mut url = if let Some(url) = base_url {
                    url
                } else {
                    return navigation::push(Screen::ServerAddressEntry)
                        .into_future(ctx)
                        .await;
                };

                url.set_path("download/torrent-contents");
                url.set_query(Some(format!("id={id}").as_ref()));
                url
            };

            match http::get(url).into_future(ctx.clone()).await {
                http::HttpOutput::Success { data, .. } => {
                    let files = data
                        .and_then(|data| serde_json::from_str::<Vec<String>>(&data).ok())
                        .map(|files| (id, files));

                    update_model(
                        &ctx,
                        PartialModel {
                            torrent_contents: Some(files),
                            ..Default::default()
                        },
                    );
                }
                http::HttpOutput::Error => {
                    // TODO: add logging
                }
            }
        }),
        DataRequest::GetMedia => Command::new(|ctx| async move {
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
                    // TODO: add logging
                }
            }
        }),
        DataRequest::GetDownloads => {
            Command::new(async move |ctx| {
                let url = {
                    let mut url = if let Some(url) = base_url {
                        url
                    } else {
                        return navigation::push(Screen::ServerAddressEntry)
                            .into_future(ctx)
                            .await;
                    };

                    url.set_path("download/get");
                    url
                };

                match http::get(url).into_future(ctx.clone()).await {
                    http::HttpOutput::Success { data, .. } => {
                        // TODO: Add logging when we can't get data or deserialize from JSON string
                        let downloads: Option<Vec<Download>> =
                            data.and_then(|data| serde_json::from_str(&data).ok());

                        update_model(
                            &ctx,
                            PartialModel {
                                downloads,
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
        DataRequest::AddDownload(download_form) => Command::new(async move |ctx| {
            let url = {
                let mut url = if let Some(url) = base_url {
                    url
                } else {
                    return navigation::push(Screen::ServerAddressEntry)
                        .into_future(ctx)
                        .await;
                };

                url.set_path("download/add");
                url
            };

            // TODO: remove unwrap
            http::post(url, serde_json::to_string(&download_form).unwrap())
                .into_future(ctx.clone())
                .await;
        }),
    }
}
