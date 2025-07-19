use crux_core::{Command, render::render};
use domain::Media;
use futures::join;
use url::Url;

use crate::{
    Effect, Event, Model, PartialModel,
    capabilities::{
        http,
        navigation::{self, Screen},
        storage,
    },
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

        let update_model_handle = ctx.spawn(|ctx| async move {
            ctx.send_event(Event::UpdateModel(PartialModel {
                base_url: Some(Some(Url::parse(&server_addres).unwrap())),
                ..Default::default()
            }));
        });
        let replace_root_handle = navigation::replace_root(Screen::List).into_future(ctx);

        join!(update_model_handle, replace_root_handle);
    })
}

pub fn handle_screen_change(model: &mut Model, screen: Screen) -> Command<Effect, Event> {
    model.current_screen = screen.clone();
    _ = render::<Effect, Event>();
    let base_url = model.base_url.clone();

    match screen {
        Screen::List => Command::new(|ctx| async move {
            let mut url = if let Some(url) = base_url {
                url
            } else {
                return navigation::push(Screen::ServerAddressEntry)
                    .into_future(ctx)
                    .await;
            };

            url.set_path("get_movies");

            match http::get(url).into_future(ctx.clone()).await {
                http::HttpOutput::Success { data, .. } => {
                    let movies =
                        data.and_then(|data| serde_json::from_str::<Vec<Media>>(&data).ok());

                    ctx.send_event(Event::UpdateModel(PartialModel {
                        media_items: Some(movies),
                        ..Default::default()
                    }));
                }
                http::HttpOutput::Error => {
                    // log
                }
            }
        }),

        Screen::Startup => Command::done(),
        Screen::ServerAddressEntry => Command::done(),
        Screen::Detail(_) => Command::done(),
        Screen::Settings => Command::done(),
        Screen::Player { .. } => Command::done(),
    }
}
