use crux_core::{Command, render::render};
use domain::{MediaContent, series::EpisodeIdentifier};
use futures::join;
use partially::Partial;
use serde::{Deserialize, Serialize};

use crate::{
    CruxContext, Effect, Event, Model,
    capabilities::{
        navigation::Screen,
        storage::{self, get_with_key_string},
    },
};

use super::utils::update_model;

#[derive(Default, Serialize, Deserialize, Partial, Clone, Debug)]
#[partially(derive(Debug, Clone, Default))]
pub struct PlaybackModel {
    pub last_position: Option<PlaybackPosition>,
    pub active_player: Option<ActivePlayerData>,
}

#[derive(Serialize, Deserialize, Partial, Clone, Debug)]
pub struct ActivePlayerData {
    pub position: PlaybackPosition,
    pub url: String,
    pub title: String,
    pub next_episode: Option<EpisodeIdentifier>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlayEvent {
    FromBeginning {
        id: String,
    },
    FromSavedPosition {
        id: String,
    },
    FromCertainEpisode {
        id: String,
        episode: EpisodeIdentifier,
    },
}

pub fn handle_playback_progress(
    model: &mut Model,
    duration_seconds: u64,
    playback_progress: PlaybackPosition,
) -> Command<Effect, Event> {
    let command =
        update_next_episode(model, duration_seconds, &playback_progress).unwrap_or(Command::done());

    command.and(Command::new(|ctx| async move {
        playback_progress.store(ctx).await;
    }))
}

fn update_next_episode(
    model: &mut Model,
    duration_seconds: u64,
    playback_progress: &PlaybackPosition,
) -> Option<Command<Effect, Event>> {
    let progress_seconds = playback_progress.get_seconds();

    if duration_seconds.saturating_sub(progress_seconds) > 30 {
        if model
            .playback
            .active_player
            .as_mut()?
            .next_episode
            .is_some()
        {
            model.playback.active_player.as_mut()?.next_episode = None;
            return Some(render());
        }
        return None;
    }

    if let PlaybackPosition::SeriesEpisode {
        id,
        episode_identifier,
        ..
    } = playback_progress
    {
        let playing_item = model
            .media_items
            .as_ref()
            .and_then(|items| items.get(id))
            .expect("Received playback progress from an unexistent series");

        let content = if let MediaContent::Series(ref series_data) = playing_item.content {
            series_data
        } else {
            unreachable!()
        };

        let next_episode = episode_identifier.find_next_episode(content)?;
        let active_player = model.playback.active_player.as_mut()?;

        if let Some(current_next_episode) = active_player.next_episode.as_ref() {
            if *current_next_episode == next_episode {
                return None;
            }
        }

        active_player.next_episode = Some(next_episode);
        return Some(render());
    }

    None
}

pub fn handle_play(model: &mut Model, play_event: PlayEvent) -> Command<Effect, Event> {
    let id = play_event.get_id().clone();
    let media_item = if let Some(item) = model.media_items.as_ref().and_then(|items| items.get(&id))
    {
        item.clone()
    } else {
        unreachable!()
    };

    let base_url_clone = model.base_url.clone().unwrap();
    let last_position = model.playback.last_position.clone();

    Command::new(|ctx| async move {
        let (initial_seconds, episode) = play_event.get_position(ctx.clone()).await;

        let playback_model = match media_item.content {
            MediaContent::Movie(content) => {
                let playback_data = PlaybackPosition::Movie {
                    id: id.clone(),
                    position_seconds: initial_seconds.unwrap_or(0),
                };

                PlaybackModel {
                    last_position,
                    active_player: Some(ActivePlayerData {
                        next_episode: None,
                        position: playback_data,
                        title: media_item.metadata.title.clone(),
                        url: base_url_clone
                            .join("static/")
                            .unwrap()
                            .join(&content.media)
                            .unwrap()
                            .to_string(),
                    }),
                }
            }
            MediaContent::Series(episodes) => {
                let defaulted_episode_id = episode.unwrap_or(
                    EpisodeIdentifier::find_earliest_available_episode(&episodes),
                );

                let season = episodes.get(&defaulted_episode_id.season_no).unwrap();
                let episode_path = season.get(&defaulted_episode_id.episode_no).unwrap();
                let title = format!(
                    "{} S{} E{}",
                    &media_item.metadata.title,
                    &defaulted_episode_id.season_no,
                    &defaulted_episode_id.episode_no
                );
                let playback_data = PlaybackPosition::SeriesEpisode {
                    id: id.clone(),
                    position_seconds: initial_seconds.unwrap_or(0),
                    episode_identifier: defaulted_episode_id,
                };

                PlaybackModel {
                    last_position,
                    active_player: Some(ActivePlayerData {
                        next_episode: None,
                        position: playback_data,
                        title,
                        url: base_url_clone
                            .join("static/")
                            .unwrap()
                            .join(episode_path.media.as_ref())
                            .unwrap()
                            .to_string(),
                    }),
                }
            }
        };

        update_model(
            &ctx,
            crate::PartialModel {
                playback: Some(playback_model),
                ..Default::default()
            },
        );

        ctx.send_event(Event::PushIfNecessary(Screen::Player));
    })
}

impl PlayEvent {
    pub async fn get_position(self, ctx: CruxContext) -> (Option<u64>, Option<EpisodeIdentifier>) {
        match self {
            Self::FromBeginning { .. } => (None, None),
            Self::FromCertainEpisode { ref id, episode } => (
                PlaybackPosition::get_series_position_from_storage(ctx, id, &episode).await,
                Some(episode),
            ),
            Self::FromSavedPosition { ref id } => {
                match PlaybackPosition::get_last_played_episode_from_storage(ctx.clone(), id).await
                {
                    Some(last_played_episode) => (
                        PlaybackPosition::get_series_position_from_storage(
                            ctx,
                            id,
                            &last_played_episode,
                        )
                        .await,
                        Some(last_played_episode),
                    ),
                    // We assume it's a movie since we haven't saved any last played episodes
                    None => (
                        PlaybackPosition::get_movie_position_from_storage(ctx, id).await,
                        None,
                    ),
                }
            }
        }
    }

    fn get_id(&self) -> &String {
        match self {
            PlayEvent::FromBeginning { id } => id,
            PlayEvent::FromSavedPosition { id } => id,
            PlayEvent::FromCertainEpisode { id, .. } => id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlaybackPosition {
    Movie {
        id: String,
        position_seconds: u64,
    },
    SeriesEpisode {
        id: String,
        episode_identifier: EpisodeIdentifier,
        position_seconds: u64,
    },
}

impl PlaybackPosition {
    pub fn get_seconds(&self) -> u64 {
        match self {
            PlaybackPosition::Movie {
                position_seconds, ..
            } => *position_seconds,
            PlaybackPosition::SeriesEpisode {
                position_seconds, ..
            } => *position_seconds,
        }
    }

    async fn store(&self, ctx: CruxContext) {
        match self {
            PlaybackPosition::Movie {
                id,
                position_seconds,
            } => {
                let storage_key = Self::get_movie_storage_key(id);
                storage::store_with_key_string(storage_key, position_seconds.to_string())
                    .into_future(ctx)
                    .await;
            }
            PlaybackPosition::SeriesEpisode {
                id,
                episode_identifier,
                position_seconds,
            } => {
                let storage_key = Self::get_series_storage_key(id, episode_identifier);
                let last_played_episode_storage_key =
                    Self::get_series_last_played_episode_storage_key(id);
                join!(
                    storage::store_with_key_string(storage_key, position_seconds.to_string())
                        .into_future(ctx.clone()),
                    storage::store_with_key_string(
                        last_played_episode_storage_key,
                        serde_json::to_string(&episode_identifier).unwrap()
                    )
                    .into_future(ctx)
                );
            }
        }
    }

    pub async fn get_movie_position_from_storage(ctx: CruxContext, id: &str) -> Option<u64> {
        let storage_key = Self::get_movie_storage_key(id);
        storage::get_with_key_string(storage_key)
            .into_future(ctx)
            .await
            .and_then(|result| result.parse().ok())
    }

    pub async fn get_series_position_from_storage(
        ctx: CruxContext,
        id: &str,
        episode_id: &EpisodeIdentifier,
    ) -> Option<u64> {
        let storage_key = Self::get_series_storage_key(id, episode_id);
        storage::get_with_key_string(storage_key)
            .into_future(ctx)
            .await
            .and_then(|result| result.parse().ok())
    }

    async fn get_last_played_episode_from_storage(
        ctx: CruxContext,
        id: &str,
    ) -> Option<EpisodeIdentifier> {
        let key = Self::get_series_last_played_episode_storage_key(id);
        get_with_key_string(key)
            .into_future(ctx)
            .await
            .and_then(|stored_value| serde_json::from_str(&stored_value).ok())
    }

    fn get_movie_storage_key(id: &str) -> String {
        format!("progress-movie-{id}")
    }

    fn get_series_storage_key(id: &str, episode_id: &EpisodeIdentifier) -> String {
        format!(
            "progress-series-{id}-{}-{}",
            episode_id.season_no, episode_id.episode_no,
        )
    }

    fn get_series_last_played_episode_storage_key(id: &str) -> String {
        format!("last-episode-{id}")
    }
}
