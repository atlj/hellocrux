use domain::{PreparingMedia, TrackSelectionItem};

use crate::State;

pub async fn handle_get_preparing_items(
    axum::extract::State(state): State,
) -> axum::Json<Vec<PreparingMedia>> {
    let (preparing, pending_track_selection) = state.preparing_list_watcher.data.borrow().clone();

    let result: Vec<PreparingMedia> = pending_track_selection
        .into_iter()
        .map(
            |(metadata, TrackSelectionItem { media, tracks })| PreparingMedia {
                media,
                metadata,
                needed_track_selection: Some(tracks),
            },
        )
        .chain(
            preparing
                .into_iter()
                .map(|(metadata, media)| PreparingMedia {
                    media,
                    metadata,
                    needed_track_selection: None,
                }),
        )
        .collect();

    axum::Json(result)
}

pub async fn handle_track_selection(
    axum::extract::State(state): State,
    axum::Json(selection): axum::Json<domain::TrackSelectionItem>,
) -> axum::response::Result<()> {
    state
        .preparing_list_watcher
        .signal_sender
        .send(crate::service::prepare::PrepareMessage::SelectTracks(
            selection,
        ))
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}
