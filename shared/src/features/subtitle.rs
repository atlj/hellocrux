use std::collections::HashMap;

use crux_core::Command;
use domain::{
    SeriesContents,
    language::LanguageCode,
    series::EpisodeIdentifier,
    subtitles::{SubtitleDownloadForm, SubtitleSearchForm, SubtitleSearchResponse},
};

use crate::{
    Event, Model, PartialModel,
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
    /// Navigate to the subtitle search configuration screen
    Search {
        media: domain::Media,
        language: LanguageCode,
        /// `None` for movies, `Some` for series episodes
        episodes: Option<Vec<EpisodeIdentifier>>,
    },
    /// Fetch subtitle search results from the server
    FetchSearchResults {
        media_id: String,
        language: LanguageCode,
        /// `None` for movies, `Some` for series episodes
        episodes: Option<Vec<EpisodeIdentifier>>,
    },
    Download {
        form: SubtitleDownloadForm,
    },
}

pub fn handle_subtitle_event(model: &Model, event: SubtitleEvent) -> crate::Command {
    match event {
        SubtitleEvent::Select { media_id, season } => {
            let Some(media) = model
                .media_items
                .get_data()
                .and_then(|data| data.get(&media_id))
                .cloned()
            else {
                return Command::done();
            };

            Command::new(async move |ctx| {
                let language = LanguageCode::English;

                let pre_selected_episodes = match (&media.content, season) {
                    (domain::MediaContent::Movie(_), _) => None,
                    (domain::MediaContent::Series(series), Some(season_no)) => {
                        let episodes = episodes_without_subtitles(series, season_no, &language)
                            .unwrap_or_default();
                        Some((season_no, episodes))
                    }
                    (domain::MediaContent::Series(_), None) => {
                        panic!("Media id belongs to a series but no season was passed")
                    }
                };

                navigation::push(Screen::SubtitleSelection {
                    media,
                    episodes: pre_selected_episodes,
                    pre_selected_language: language,
                })
                .into_future(ctx)
                .await;
            })
        }

        SubtitleEvent::Search {
            media,
            language,
            episodes,
        } => Command::new(|ctx| async move {
            navigation::push(Screen::SubtitleSearchResult {
                media,
                language,
                episodes,
            })
            .into_future(ctx)
            .await
        }),

        SubtitleEvent::FetchSearchResults {
            media_id,
            language,
            episodes,
        } => fetch_subtitle_results(model, media_id, language, episodes),

        SubtitleEvent::Download { form } => download_subtitles(model, form).then(
            Command::new(async |ctx| {
                navigation::reset(Some(Screen::List)).into_future(ctx).await;
            })
            .and(Command::event(Event::UpdateData(DataRequest::GetMedia))),
        ),
    }
}

/// Returns the episode identifiers in `season_no` that are missing a subtitle
/// for the given `language`. Returns `None` if the season does not exist.
fn episodes_without_subtitles(
    series: &SeriesContents,
    season_no: u32,
    language: &LanguageCode,
) -> Option<Vec<u32>> {
    let season = series.get(&season_no)?;
    Some(
        season
            .iter()
            .filter(|(_, paths)| {
                !paths
                    .subtitle_paths
                    .iter()
                    .any(|subtitle| subtitle.language == *language)
            })
            .map(|(episode_no, _)| *episode_no)
            .collect(),
    )
}

fn fetch_subtitle_results(
    model: &Model,
    media_id: String,
    language: LanguageCode,
    episodes: Option<Vec<EpisodeIdentifier>>,
) -> crate::Command {
    let subtitles_search_endpoint = {
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
        episode_identifiers: episodes.clone(),
    };

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

        let output = http::post(
            subtitles_search_endpoint,
            serde_json::to_string(&form).expect("Form to be valid JSON"),
        )
        .into_future(ctx.clone())
        .await;

        let result = match output
            .into_option()
            .and_then(|result_string| {
                serde_json::from_str::<SubtitleSearchResponse>(&result_string).ok()
            })
            .map(|search_response| -> SubtitleSearchResults {
                let episode_results = match episodes {
                    Some(ref identifiers) => search_response
                        .into_iter()
                        .zip(identifiers.iter().cloned())
                        .map(|(results, identifier)| {
                            (
                                identifier,
                                results
                                    .into_iter()
                                    .map(SubtitleSearchResult::from)
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect(),
                    None => HashMap::new(),
                };
                SubtitleSearchResults {
                    media_id,
                    language,
                    episode_results,
                }
            }) {
            Some(results) => QueryState::Success { data: results },
            None => QueryState::Error {
                message: "Couldn't search subtitles due to server error".to_string(),
            },
        };

        update_model(
            &ctx,
            PartialModel {
                subtitles_search_results: Some(result),
                ..Default::default()
            },
        );
    })
}

fn download_subtitles(model: &Model, form: SubtitleDownloadForm) -> crate::Command {
    let subtitles_download_endpoint = {
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
            serde_json::to_string(&form).expect("Form to be serializable"),
        )
        .into_future(ctx.clone())
        .await;

        let download_result = match result {
            http::HttpOutput::Success { .. } => QueryState::Success { data: () },
            http::HttpOutput::Error => QueryState::Error {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use domain::{
        MediaPaths, SeasonContents, SeriesContents, language::LanguageCode, subtitles::SubtitlePath,
    };

    use super::episodes_without_subtitles;

    fn episode_with_subtitle(language: LanguageCode) -> MediaPaths {
        MediaPaths {
            media: String::new(),
            track_name: String::new(),
            subtitle_paths: Box::new([SubtitlePath {
                language,
                srt_path: String::new(),
                track_path: String::new(),
            }]),
        }
    }

    fn series_with_season(season_no: u32, season: SeasonContents) -> SeriesContents {
        let mut series = HashMap::new();
        series.insert(season_no, season);
        series
    }

    #[test]
    fn returns_none_for_missing_season() {
        let series: SeriesContents = HashMap::new();
        assert!(episodes_without_subtitles(&series, 1, &LanguageCode::English).is_none());
    }

    #[test]
    fn returns_empty_when_all_episodes_are_subtitled() {
        let mut season: SeasonContents = HashMap::new();
        season.insert(1, episode_with_subtitle(LanguageCode::English));
        season.insert(2, episode_with_subtitle(LanguageCode::English));
        let series = series_with_season(1, season);

        let result = episodes_without_subtitles(&series, 1, &LanguageCode::English).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn ignores_subtitles_in_other_languages() {
        let mut season: SeasonContents = HashMap::new();
        season.insert(1, episode_with_subtitle(LanguageCode::Turkish));
        let series = series_with_season(1, season);

        let result = episodes_without_subtitles(&series, 1, &LanguageCode::English).unwrap();
        assert_eq!(result, vec![1]);
    }
}
