use std::{collections::HashMap, path::Path};

use crux_core::Command;
use domain::{
    Download, DownloadForm, Media,
    series::{EditSeriesFileMappingForm, EpisodeIdentifier, file_mapping_form_state},
};

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
    SetSeriesFileMapping(EditSeriesFileMappingForm<file_mapping_form_state::NeedsValidation>),
}

pub fn update_data(model: &mut Model, request: DataRequest) -> Command<Effect, Event> {
    let base_url = model.base_url.clone();

    match request {
        DataRequest::SetSeriesFileMapping(form) => {
            // TODO remove all unwraps
            let (current_id, files_list) = model.torrent_contents.as_ref().unwrap();

            assert_eq!(*current_id, *form.id);
            let validated_form = form
                .validate(
                    // TODO prevent cloning here
                    files_list.keys().cloned().collect::<Box<[_]>>().as_ref(),
                )
                .unwrap();

            Command::new(|ctx| async move {
                // TODO add validation
                let url = {
                    let mut url = if let Some(url) = base_url {
                        url
                    } else {
                        return navigation::push(Screen::ServerAddressEntry)
                            .into_future(ctx)
                            .await;
                    };

                    url.set_path("download/set-file-mapping");
                    url
                };

                // TODO: remove unwrap
                http::post(url, serde_json::to_string(&validated_form).unwrap())
                    .into_future(ctx.clone())
                    .await;
            })
        }
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
                        .map(|files| {
                            let len = files.len();
                            let map = files.into_iter().fold(
                                HashMap::with_capacity(len),
                                |mut map, file| {
                                    if let Some(extension) = (file.as_ref() as &Path).extension() {
                                        // TODO make this a generic check
                                        if extension == "srt" {
                                            return map;
                                        }
                                    }

                                    let identifier = detect_episode_identifier(&file).unwrap_or(
                                        EpisodeIdentifier {
                                            season_no: 1,
                                            episode_no: 1,
                                        },
                                    );
                                    map.insert(file, identifier);
                                    map
                                },
                            );
                            (id, map)
                        });

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

fn detect_episode_identifier(path: &str) -> Option<EpisodeIdentifier> {
    let re = regex::Regex::new(r".*S([0-9]+)E([0-9]+).*").expect("Invalid regex supplied");
    let first_capture = re.captures_iter(path).next()?;
    let (_, [season_no_str, episode_no_str]) = first_capture.extract();

    Some(EpisodeIdentifier {
        season_no: season_no_str.parse().ok()?,
        episode_no: episode_no_str.parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use domain::series::EpisodeIdentifier;

    use crate::features::data::detect_episode_identifier;

    #[test]
    fn test_detect_episode_identifier() {
        assert_eq!(
            detect_episode_identifier("my-series.S02E01.1080p.x265-HEYYY.mkv").unwrap(),
            EpisodeIdentifier {
                season_no: 2,
                episode_no: 1
            }
        );

        assert!(detect_episode_identifier("my-series.S02E.1080p.x265-HEYYY.mkv").is_none());
        assert!(detect_episode_identifier("my-series.S02.1080p.x265-HEYYY.mkv").is_none());
        assert!(detect_episode_identifier("my-series.02.1080p.x265-HEYYY.mkv").is_none());
        assert!(detect_episode_identifier("my-series.E2S7.1080p.x265-HEYYY.mkv").is_none());
        assert!(detect_episode_identifier("my-series.SE.1080p.x265-HEYYY.mkv").is_none());
    }
}
