use crate::State;

pub async fn handle_get_preparing_items(
    axum::extract::State(state): State,
) -> axum::Json<(
    Vec<domain::MediaIdentifier>,
    Vec<domain::TrackSelectionItem>,
)> {
    let (preparing, pending_track_selection) = state.preparing_list_watcher.data.borrow().clone();

    axum::Json((preparing, pending_track_selection))
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
