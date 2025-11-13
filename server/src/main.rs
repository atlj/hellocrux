use axum::{
    Json, Router, extract,
    routing::{get, post},
};
use clap::Parser;
use domain::Media;
use download_handlers::watch_and_process_downloads;
use log::info;
use server::{AppState, Args, State, download_handlers, media::watch_media_items};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    let download_path = {
        let mut download_path = args.media_dir.clone();
        download_path.push("qbittorrent");
        download_path
    };

    let (media_update_request_sender, media_list_receiver, media_watcher_join_handler) =
        watch_media_items(args.media_dir.clone()).await;

    let (download_sender, value_receiver, bittorrent_client_join_handle) =
        download_handlers::spawn_download_event_loop(download_path).await;

    let torrent_watcher_handle = {
        let media_dir_clone = args.media_dir.clone();
        let receiver_clone = value_receiver.clone();
        let sender_clone = download_sender.clone();
        let media_update_request_sender_clone = media_update_request_sender.clone();
        tokio::spawn(watch_and_process_downloads(
            media_dir_clone,
            receiver_clone,
            sender_clone,
            media_update_request_sender_clone,
        ))
    };

    let shared_state = AppState {
        media_channels: (media_update_request_sender, media_list_receiver),
        download_channels: (download_sender, value_receiver),
    };

    let app = Router::new()
        .nest_service("/static", ServeDir::new(args.media_dir))
        .route("/health", get(health_handler))
        .route("/get_movies", get(movie_list_handler))
        .route("/download/add", post(download_handlers::add_download))
        .route("/download/remove", post(download_handlers::remove_download))
        .route(
            "/download/torrent-contents",
            get(download_handlers::get_torrent_contents),
        )
        .route(
            "/download/set-file-mapping",
            post(download_handlers::update_file_mapping),
        )
        .route("/download/get", get(download_handlers::get_downloads))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Server can't be started");

    info!("Starting the server on port 3000");

    axum::serve(listener, app).await.unwrap();

    info!("Killing the server");

    torrent_watcher_handle.abort();
    bittorrent_client_join_handle.abort();
    media_watcher_join_handler.abort();
}

async fn movie_list_handler(extract::State(state): State) -> Json<Box<[Media]>> {
    Json(state.media_channels.1.borrow().as_ref().into())
}

async fn health_handler() -> String {
    "alive".to_string()
}
