use crux_core::Command;
use domain::MediaContent;
use serde::{Deserialize, Serialize};

use crate::{
    Effect, Event, Model,
    capabilities::{
        navigation::{self, Screen},
        storage::{get_with_key_string, store_with_key_string},
    },
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlayEvent {
    FromStart { id: String },
    FromLastPosition { id: String },
    FromCertainEpisode { id: String, episode: Episode },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaybackProgressData {
    id: String,
    episode: Option<Episode>,
    progress_seconds: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Episode {
    pub season: u32,
    pub episode: u32,
}

impl Default for Episode {
    fn default() -> Self {
        Episode {
            season: 1,
            episode: 1,
        }
    }
}

pub fn handle_playback_progress(
    playback_progress_data: PlaybackProgressData,
) -> Command<Effect, Event> {
    Command::new(|ctx| async move {
        let key = format!("progress-{}", playback_progress_data.id);

        store_with_key_string(key, serde_json::to_string(&playback_progress_data).unwrap())
            .into_future(ctx)
            .await;
    })
}

pub fn handle_play(model: &mut Model, play_event: PlayEvent) -> Command<Effect, Event> {
    let play_event_clone = play_event.clone();

    let (id, episode) = match play_event {
        PlayEvent::FromStart { id } => (id, None),
        PlayEvent::FromLastPosition { id } => (id, None),
        PlayEvent::FromCertainEpisode { id, episode } => (id, Some(episode)),
    };

    let item = model
        .media_items
        .as_ref()
        .and_then(|items| items.iter().find(|item| item.id == id));

    let base_url = model.base_url.clone();

    match item {
        Some(item) => {
            let url = match item.content {
                MediaContent::Movie(ref content) => base_url
                    .unwrap()
                    .join("static/")
                    .unwrap()
                    .join(content)
                    .unwrap(),
                MediaContent::Series(ref episodes) => {
                    let episode = episode.clone().unwrap_or_default();

                    let season = episodes.get(&episode.season).unwrap();
                    let episode = season.get(&episode.episode).unwrap();

                    base_url
                        .unwrap()
                        .join("static/")
                        .unwrap()
                        .join(episode)
                        .unwrap()
                }
            };

            Command::new(|ctx| async move {
                let initial_seconds: Option<u64> = match play_event_clone {
                    PlayEvent::FromStart { .. } => None,
                    PlayEvent::FromCertainEpisode { .. } => None,
                    PlayEvent::FromLastPosition { id } => {
                        let key = format!("progress-{}", id);
                        let storage_string =
                            get_with_key_string(key).into_future(ctx.clone()).await;
                        match storage_string {
                            Some(ref storage_string) => {
                                let progress_data =
                                    serde_json::from_str::<PlaybackProgressData>(storage_string)
                                        .unwrap();
                                Some(progress_data.progress_seconds)
                            }
                            None => None,
                        }
                    }
                };

                navigation::push(Screen::Player {
                    id,
                    episode,
                    initial_seconds,
                    url: url.to_string(),
                })
                .into_future(ctx)
                .await;
            })
        }
        None => todo!(),
    }
}
