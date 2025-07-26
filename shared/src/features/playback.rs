use crux_core::Command;
use domain::MediaContent;
use futures::join;
use partially::Partial;
use serde::{Deserialize, Serialize};

use crate::{
    CruxContext, Effect, Event, Model,
    capabilities::{
        navigation::{self, Screen},
        storage::{get_with_key_string, store_with_key_string},
    },
};

#[derive(Default, Serialize, Deserialize, Partial, Clone, Debug)]
#[partially(derive(Debug, Clone, Default))]
pub struct PlaybackModel {
    pub last_position: PlaybackProgressData,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlayEvent {
    FromStart { id: String },
    FromLastPosition { id: String },
    FromCertainEpisode { id: String, episode: Episode },
}

pub fn handle_playback_progress(
    playback_progress_data: PlaybackProgressData,
) -> Command<Effect, Event> {
    Command::new(|ctx| async move { playback_progress_data.store(ctx).await })
}

pub fn handle_play(model: &mut Model, play_event: PlayEvent) -> Command<Effect, Event> {
    let id = play_event.get_id().clone();
    let item = if let Some(item) = model
        .media_items
        .as_ref()
        .and_then(|items| items.iter().find(|item| item.id == id))
    {
        item.clone()
    } else {
        unreachable!()
    };
    let base_url_clone = model.base_url.clone().unwrap();

    Command::new(|ctx| async move {
        let (initial_seconds, episode) = play_event.get_position(ctx.clone()).await;
        let url = match item.content {
            MediaContent::Movie(content) => base_url_clone
                .join("static/")
                .unwrap()
                .join(&content)
                .unwrap(),
            MediaContent::Series(episodes) => {
                let defaulted_episode = match episode {
                    Some(ref episode) => episode,
                    None => &Episode::default(),
                };

                let season = episodes.get(&defaulted_episode.season).unwrap();
                let episode = season.get(&defaulted_episode.episode).unwrap();

                base_url_clone
                    .join("static/")
                    .unwrap()
                    .join(episode)
                    .unwrap()
            }
        };

        // TODO, only push url and initial seconds
        navigation::push(Screen::Player {
            id: id.clone(),
            episode,
            initial_seconds,
            url: url.to_string(),
        })
        .into_future(ctx)
        .await;
    })
}

impl PlayEvent {
    async fn get_position(&self, ctx: CruxContext) -> (Option<u64>, Option<Episode>) {
        match self {
            Self::FromStart { .. } => (None, None),
            Self::FromLastPosition { id } => {
                let last_episode = PlaybackProgressData::get_last_episode(ctx.clone(), id).await;
                (
                    PlaybackProgressData::get_from_storage(ctx, id, last_episode.as_ref()).await,
                    last_episode,
                )
            }
            Self::FromCertainEpisode { id, episode } => (
                PlaybackProgressData::get_from_storage(ctx, id, Some(episode)).await,
                Some(episode.clone()),
            ),
        }
    }

    fn get_id(&self) -> &String {
        match self {
            PlayEvent::FromStart { id } => id,
            PlayEvent::FromLastPosition { id } => id,
            PlayEvent::FromCertainEpisode { id, .. } => id,
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaybackProgressData {
    pub id: String,
    pub episode: Option<Episode>,
    pub progress_seconds: u64,
}

impl PlaybackProgressData {
    async fn store(&self, ctx: CruxContext) {
        let key = Self::get_key(&self.id, self.episode.as_ref());
        let store_self_future =
            store_with_key_string(key, self.progress_seconds.to_string()).into_future(ctx.clone());

        match self.episode {
            None => {
                store_self_future.await;
            }
            Some(_) => {
                join!(store_self_future, self.store_last_episode(ctx.clone()));
            }
        };
    }

    pub async fn get_from_storage(
        ctx: CruxContext,
        id: &str,
        episode: Option<&Episode>,
    ) -> Option<u64> {
        let key = Self::get_key(id, episode);
        get_with_key_string(key)
            .into_future(ctx)
            .await
            .and_then(|value| value.parse().ok())
    }

    async fn store_last_episode(&self, ctx: CruxContext) {
        let key = format!("last-episode-{}", self.id);
        store_with_key_string(key, serde_json::to_string(&self.episode).unwrap())
            .into_future(ctx)
            .await;
    }

    pub async fn get_last_episode(ctx: CruxContext, id: &str) -> Option<Episode> {
        let key = format!("last-episode-{id}");
        get_with_key_string(key)
            .into_future(ctx)
            .await
            .and_then(|stored_value| serde_json::from_str(&stored_value).ok())
    }

    fn get_key(id: &str, episode: Option<&Episode>) -> String {
        match episode {
            None => format!("progress-movie-{id}"),
            Some(episode) => format!(
                "progress-series-{id}-{}-{}",
                episode.season, episode.episode
            ),
        }
    }
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
