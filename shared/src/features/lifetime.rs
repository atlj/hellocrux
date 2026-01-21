use crux_core::{Command, render::render};
use domain::Media;
use url::Url;

use crate::{
    Effect, Event, Model, PartialModel,
    capabilities::{
        navigation::{self, Screen},
        service_discovery, storage,
    },
    features::utils::update_model,
};

use super::{
    data::DataRequest,
    playback::{PlayEvent, PlaybackModel, PlaybackPosition},
};

pub fn handle_startup(_: &mut Model) -> Command<Effect, Event> {
    Command::new(|ctx| async move {
        let server_addres = if let Some(address) = storage::get("server_address")
            .into_future(ctx.clone())
            .await
        {
            address
        } else {
            return navigation::replace_root(Screen::ServerAddressEntry)
                .into_future(ctx)
                .await;
        };

        update_model(
            &ctx,
            PartialModel {
                base_url: Some(Some(Url::parse(&server_addres).unwrap())),
                ..Default::default()
            },
        );

        navigation::replace_root(Screen::List)
            .into_future(ctx)
            .await;
    })
}

pub fn handle_screen_change(model: &mut Model, screen: Screen) -> Command<Effect, Event> {
    model.current_screen = screen.clone();

    let command = match screen {
        Screen::List => Command::event(Event::UpdateData(DataRequest::GetMedia)),
        Screen::Detail(Media { id, .. }) => Command::new(|ctx| async move {
            let (initial_seconds, episode) = PlayEvent::FromSavedPosition { id: id.clone() }
                .get_position(ctx.clone())
                .await;

            let position = initial_seconds.map(|position_seconds| match episode {
                None => PlaybackPosition::Movie {
                    id,
                    position_seconds,
                },
                Some(episode_identifier) => PlaybackPosition::SeriesEpisode {
                    id,
                    episode_identifier,
                    position_seconds,
                },
            });

            update_model(
                &ctx,
                PartialModel {
                    playback: Some(PlaybackModel {
                        last_position: position,
                        active_player: None,
                    }),
                    ..Default::default()
                },
            );
        }),
        Screen::ServerDownloads => Command::done(),
        Screen::AddDownload => Command::done(),
        Screen::Startup => Command::done(),
        Screen::ServerAddressEntry => {
            Command::new(|ctx| async move {
                // Stopped when connection is successful
                service_discovery::start().into_future(ctx).await
            })
        }
        Screen::ServerFileMapping(id) => match model.torrent_contents {
            Some((ref existing_id, _)) if *existing_id == id => Command::done(),
            _ => Command::event(Event::UpdateData(DataRequest::GetContents(id))),
        },
        Screen::Settings => Command::done(),
        Screen::Player => Command::done(),
    };

    render().and(command)
}
