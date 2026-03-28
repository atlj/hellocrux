use domain::series::{EditSeriesFileMappingForm, file_mapping_form_state::NeedsValidation};

use crate::capabilities::{
    http,
    navigation::{self, Screen},
};

pub fn handle_file_mapping(
    model: &crate::Model,
    form: EditSeriesFileMappingForm<NeedsValidation>,
) -> crate::Command {
    // TODO remove all unwraps
    let (current_id, files_list) = model.torrent_contents.as_ref().unwrap();

    assert_eq!(*current_id, *form.id);
    let validated_form = form
        .validate(
            // TODO prevent cloning here
            files_list.keys().cloned().collect::<Box<[_]>>().as_ref(),
        )
        .unwrap();

    let base_url = model.base_url.clone();

    crate::Command::new(|ctx| async move {
        // TODO add validation
        let url = {
            let mut url = if let Some(url) = base_url {
                url
            } else {
                return navigation::push(Screen::ServerAddressEntry)
                    .into_future(ctx)
                    .await;
            };

            url.set_path("download/set-file-mapping");
            url
        };

        // TODO: remove unwrap
        http::post(url, serde_json::to_string(&validated_form).unwrap())
            .into_future(ctx.clone())
            .await;
    })
}
