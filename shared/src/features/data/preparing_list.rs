use domain::PreparingMedia;

use crate::{
    Model, PartialModel,
    capabilities::{
        http,
        navigation::{self, Screen},
    },
    features::utils::update_model,
};

pub fn handle_get_preparing(model: &Model) -> crate::Command {
    let base_url = model.base_url.clone();

    crate::Command::new(async move |ctx| {
        let url = {
            let mut url = if let Some(url) = base_url {
                url
            } else {
                return navigation::push(Screen::ServerAddressEntry)
                    .into_future(ctx)
                    .await;
            };

            url.set_path("prepare/get");
            url
        };

        match http::get(url).into_future(ctx.clone()).await {
            http::HttpOutput::Success { data, .. } => {
                // TODO: Add logging when we can't get data or deserialize from JSON string
                let preparing_list: Option<Vec<PreparingMedia>> =
                    data.and_then(|data| serde_json::from_str(&data).ok());

                update_model(
                    &ctx,
                    PartialModel {
                        preparing_list,
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

// pub fn handle_send_track_selection(model: &Model, download_form: DownloadForm) -> crate::Command {
//     todo!("Implement me!");
//     let base_url = model.base_url.clone();
//
//     crate::Command::new(async move |ctx| {
//         let url = {
//             let mut url = if let Some(url) = base_url {
//                 url
//             } else {
//                 return navigation::push(Screen::ServerAddressEntry)
//                     .into_future(ctx)
//                     .await;
//             };
//
//             url.set_path("download/add");
//             url
//         };
//
//         // TODO: remove unwrap
//         http::post(url, serde_json::to_string(&download_form).unwrap())
//             .into_future(ctx.clone())
//             .await;
//     })
// }
