use std::collections::HashMap;

use crux_core::Command;
use domain::{language::LanguageCode, subtitles::SubtitleDownloadForm};

use crate::{
    Effect, Event, Model, PartialModel,
    capabilities::{
        http,
        navigation::{self, Screen},
    },
    features::{
        data::DataRequest,
        query::{
            QueryState,
            view_model_queries::{SubtitleSearchResult, SubtitleSearchResults},
        },
        utils::update_model,
    },
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum SubtitleEvent {
    Select {
        media_id: String,
        season: Option<u32>,
    },
    Search {
        media_id: String,
        language: LanguageCode,
        episodes: Option<(u32, Vec<u32>)>,
    },
    Download {
        form: SubtitleDownloadForm,
    },
}

pub fn handle_subtitle_event(model: &Model, event: SubtitleEvent) -> Command<Effect, Event> {
    match event {
        SubtitleEvent::Select { media_id, season } => {
            // TODO extract
            let media = model
                .media_items
                .get_data()
                .and_then(|data| data.get(&media_id))
                .expect("Media id to point to a valid media item")
                .clone();

            Command::new(async move |ctx| {
                let language = LanguageCode::English;

                let pre_selected_episodes = match &media.content {
                    domain::MediaContent::Movie(_media_paths) => todo!(),
                    domain::MediaContent::Series(episodes) => {
                        let season_data = episodes.get(&season.unwrap()).unwrap();
                        season_data
                            .iter()
                            .filter(|episode| {
                                !episode.1.subtitles.iter().any(|subtitle| {
                                    TryInto::<LanguageCode>::try_into(
                                        subtitle.language_iso639_2t.as_str(),
                                    )
                                    .unwrap()
                                        == language
                                })
                            })
                            .map(|episode| *episode.0)
                            .collect()
                    }
                };

                navigation::push(navigation::Screen::SubtitleSelection {
                    media,
                    season: season.unwrap(),
                    pre_selected_episodes,
                    pre_selected_language: language,
                })
                .into_future(ctx)
                .await;
            })
        }

        SubtitleEvent::Search {
            media_id,
            language,
            episodes,
        } => Command::new(|ctx| async move {
            navigation::push(navigation::Screen::SubtitleSearchResult {
                media_id,
                language,
                episodes,
            })
            .into_future(ctx)
            .await
        }),
        SubtitleEvent::Download { form } => download_subtitles(model, form).then(
            Command::new(async |ctx| {
                navigation::reset(Some(Screen::List)).into_future(ctx).await;
            })
            .and(Command::event(Event::UpdateData(DataRequest::GetMedia))),
        ),
    }
}

pub fn search_subtitles(
    model: &Model,
    media_id: String,
    language: domain::language::LanguageCode,
    episodes: Option<(u32, Vec<u32>)>,
) -> Command<Effect, Event> {
    let subtitles_search_endpoint = {
        // TODO remove expect
        let mut url = model
            .base_url
            .clone()
            .expect("Base url to be defined at this stage");
        url.set_path("subtitles/search");
        url
    };
    let previous_search_results = model.subtitles_search_results.get_data().cloned();

    Command::new(|ctx| async move {
        update_model(
            &ctx,
            PartialModel {
                subtitles_search_results: Some(QueryState::Loading {
                    data: previous_search_results,
                }),
                ..Default::default()
            },
        );
        // TODO remove unwrap here
        let episodes = episodes.unwrap();

        let urls = episodes.1.into_iter().map(|episode| {
            let mut url = subtitles_search_endpoint.clone();

            let query = domain::subtitles::SearchSubtitlesQuery {
                media_id: media_id.clone(),
                language_code: language.clone(),
                season_no: Some(episodes.0),
                episode_no: Some(episode),
            };
            let query_string = serde_urlencoded::to_string(query).unwrap();

            url.set_query(Some(&query_string));
            (episode, url)
        });

        let futures = urls.map(|(episode, url)| {
            let ctx = ctx.clone();
            async move { (episode, http::get(url).into_future(ctx).await) }
        });

        let results = futures::future::join_all(futures).await;
        let episode_results: HashMap<u32, Vec<SubtitleSearchResult>> = results
            .into_iter()
            .filter_map(|(episode, result)| match result {
                http::HttpOutput::Success { data, .. } => data.map(|data| (episode, data)),
                http::HttpOutput::Error => None,
            })
            .filter_map(|(episode, result_string)| {
                serde_json::from_str::<Vec<subtitles::SubtitleDownloadOption<usize>>>(
                    &result_string,
                )
                .ok()
                .map(|options| (episode, options))
            })
            .fold(HashMap::with_capacity(20), |mut map, (episode, options)| {
                map.insert(
                    episode,
                    options
                        .into_iter()
                        .map(|download_option| download_option.into())
                        .collect(),
                );
                map
            });

        let search_results = SubtitleSearchResults {
            media_id,
            language,
            season: episodes.0,
            episode_results,
        };

        update_model(
            &ctx,
            PartialModel {
                subtitles_search_results: Some(QueryState::Success {
                    data: search_results,
                }),
                ..Default::default()
            },
        );
    })
}

fn download_subtitles(model: &Model, form: SubtitleDownloadForm) -> Command<Effect, Event> {
    let subtitles_download_endpoint = {
        // TODO remove expect
        let mut url = model
            .base_url
            .clone()
            .expect("Base url to be defined at this stage");
        url.set_path("subtitles/download");
        url
    };

    Command::new(|ctx| async move {
        update_model(
            &ctx,
            PartialModel {
                subtitle_download_results: Some(Some(QueryState::Loading { data: None })),
                ..Default::default()
            },
        );

        let result = http::post(
            subtitles_download_endpoint,
            serde_json::to_string(&form).expect("Form to be serializible"),
        )
        .into_future(ctx.clone())
        .await;

        let download_result = match result {
            http::HttpOutput::Success { .. } => QueryState::Success { data: () },
            http::HttpOutput::Error => QueryState::Error {
                // TODO: better error message
                message: "Couldn't download subtitles".to_string(),
            },
        };

        update_model(
            &ctx,
            PartialModel {
                subtitle_download_results: Some(Some(download_result)),
                ..Default::default()
            },
        );
    })
}
