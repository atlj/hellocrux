use domain::{Download, DownloadForm};

use crate::{
    Model, PartialModel,
    capabilities::{
        http,
        navigation::{self, Screen},
    },
    features::utils::update_model,
};

pub fn handle_get_downloads(model: &Model) -> crate::Command {
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

            url.set_path("download/get");
            url
        };

        match http::get(url).into_future(ctx.clone()).await {
            http::HttpOutput::Success { data, .. } => {
                // TODO: Add logging when we can't get data or deserialize from JSON string
                let downloads: Option<Vec<Download>> =
                    data.and_then(|data| serde_json::from_str(&data).ok());

                update_model(
                    &ctx,
                    PartialModel {
                        downloads,
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

pub fn handle_add_download(model: &Model, download_form: DownloadForm) -> crate::Command {
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

            url.set_path("download/add");
            url
        };

        // TODO: remove unwrap
        http::post(url, serde_json::to_string(&download_form).unwrap())
            .into_future(ctx.clone())
            .await;
    })
}
