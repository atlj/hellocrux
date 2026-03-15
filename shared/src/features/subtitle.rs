use crux_core::Command;
use domain::{language::LanguageCode, subtitles::SubtitleDownloadForm};

use crate::{Effect, Event, Model, capabilities::navigation};

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
        SubtitleEvent::Download { form: _ } => todo!(),
    }
}
