use std::sync::Arc;

use axum::{
    Json, Router, extract,
    routing::{get, post},
};
use clap::Parser;
use domain::Media;
use download_handlers::watch_and_process_downloads;
use log::info;
use server::{AppState, Args, State, download_handlers, media::get_media_items};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    tokio::fs::create_dir_all(&args.media_dir)
        .await
        .expect("Couldn't create media dir");

    info!("Crawling media items");

    let entries: Arc<[Media]> = get_media_items(args.media_dir.clone()).await.into();

    info!("Found {:#?} media items", entries.len());

    let download_path = {
        let mut download_path = args.media_dir.clone();
        download_path.push("qbittorrent");
        download_path
    };

    let (sender, value_receiver, join_handle) =
        download_handlers::spawn_download_event_loop(download_path).await;

    let torrent_watcher_handle = {
        let media_dir_clone = args.media_dir.clone();
        let receiver_clone = value_receiver.clone();
        let sender_clone = sender.clone();
        tokio::spawn(watch_and_process_downloads(
            media_dir_clone,
            receiver_clone,
            sender_clone,
        ))
    };

    let shared_state = AppState {
        entries,
        download_channels: (sender, value_receiver),
    };

    let app = Router::new()
        .nest_service("/static", ServeDir::new(args.media_dir))
        .route("/health", get(health_handler))
        .route("/get_movies", get(movie_list_handler))
        .route("/download/add", post(download_handlers::add_download))
        .route("/download/remove", post(download_handlers::remove_download))
        .route("/download/get", get(download_handlers::get_downloads))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Server can't be started");

    info!("Starting the server on port 3000");

    axum::serve(listener, app).await.unwrap();

    torrent_watcher_handle.abort();
    join_handle.abort();
}

async fn movie_list_handler(extract::State(state): State) -> Json<Box<[Media]>> {
    Json(state.entries.as_ref().into())
}

async fn health_handler() -> String {
    "alive".to_string()
}
