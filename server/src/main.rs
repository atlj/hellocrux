use axum::{
    Json, Router, extract,
    routing::{get, post},
};
use clap::Parser;
use domain::Media;
use download_handlers::watch_and_process_downloads;
use log::info;
use server::{AppState, Args, State, download_handlers};
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

    let (media_signal_watcher, media_signal_receiver): (
        server::service::media::MediaSignalWatcher,
        _,
    ) = server::watch::new_watcher_receiver_pair(Box::new([]));
    let (download_signal_watcher, download_signal_receiver): (
        server::service::torrent::DownloadSignalWatcher,
        _,
    ) = server::watch::new_watcher_receiver_pair(Box::new([]));
    let processing_list_watcher = server::ProcessingListWatcher::new(Box::new([]));
    let shared_state = AppState {
        media_signal_watcher,
        download_signal_watcher,
        processing_list_watcher,
    };

    let abort_services = {
        let media_watcher_join_handler = server::service::media::spawn(
            args.media_dir.clone(),
            media_signal_receiver,
            shared_state.media_signal_watcher.clone(),
        )
        .await;

        let bittorrent_client_join_handle = server::service::torrent::spawn(
            download_path,
            download_signal_receiver,
            shared_state.download_signal_watcher.clone(),
        )
        .await;

        let torrent_watcher_handle = {
            tokio::spawn(watch_and_process_downloads(
                args.media_dir.clone(),
                shared_state.clone(),
            ))
        };

        move || {
            media_watcher_join_handler.abort();
            bittorrent_client_join_handle.abort();
            torrent_watcher_handle.abort();
        }
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

    abort_services()
}

async fn movie_list_handler(extract::State(state): State) -> Json<Box<[Media]>> {
    Json(state.media_signal_watcher.data.borrow().as_ref().into())
}

async fn health_handler() -> String {
    "alive".to_string()
}
