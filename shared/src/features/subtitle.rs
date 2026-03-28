use std::collections::HashMap;

use crux_core::Command;
use domain::{
    language::LanguageCode,
    series::EpisodeIdentifier,
    subtitles::{SubtitleDownloadForm, SubtitleSearchForm, SubtitleSearchResponse},
};

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
                                !episode
                                    .1
                                    .subtitle_paths
                                    .iter()
                                    .any(|subtitle| subtitle.language == language)
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
    episode_identifiers: Option<Vec<EpisodeIdentifier>>,
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

    let form = SubtitleSearchForm {
        media_id: media_id.clone(),
        language_code: language.clone(),
        episode_identifiers: episode_identifiers.clone(),
    };

    Command::new(|ctx| async move {
        // 1. Set the state to loading
        update_model(
            &ctx,
            PartialModel {
                subtitles_search_results: Some(QueryState::Loading {
                    data: previous_search_results,
                }),
                ..Default::default()
            },
        );

        // 2. Request from server
        // TODO remove expect
        let output = http::post(
            subtitles_search_endpoint,
            serde_json::to_string(&form).expect("Form to be valid JSON"),
        )
        .into_future(ctx.clone())
        .await;

        // TODO remove nesting and unwraps
        let result = match output
            .into_option()
            .and_then(|result_string| {
                serde_json::from_str::<SubtitleSearchResponse>(&result_string).ok()
            })
            .map(|search_response| -> SubtitleSearchResults {
                let season = episode_identifiers
                    .clone()
                    .unwrap()
                    .first()
                    .unwrap()
                    .season_no;

                let episode_results: HashMap<_, _> = search_response
                    .into_iter()
                    .zip(episode_identifiers.clone().unwrap())
                    .map(|(results, episode_identifier)| {
                        (
                            episode_identifier.episode_no,
                            results
                                .into_iter()
                                .map(SubtitleSearchResult::from)
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect();
                SubtitleSearchResults {
                    media_id,
                    language,
                    // TODO replace
                    season,
                    episode_results,
                }
            }) {
            Some(results) => QueryState::Success { data: results },
            None => QueryState::Error {
                message: "Couldn't search subtitles due to server error".to_string(),
            },
        };

        // 3. Set the data
        update_model(
            &ctx,
            PartialModel {
                subtitles_search_results: Some(result),
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
