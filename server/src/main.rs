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
use torrent::qbittorrent_client::{QBittorrentClient, QBittorrentClientMessage};
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

    let (media_signal_watcher, media_signal_receiver): (server::MediaSignalWatcher, _) =
        server::watch::new_watcher_receiver_pair(Box::new([]));
    let (download_signal_watcher, download_signal_receiver): (server::DownloadSignalWatcher, _) =
        server::watch::new_watcher_receiver_pair(Box::new([]));
    let processing_list_watcher = server::ProcessingListWatcher::new(Box::new([]));
    let shared_state = AppState {
        media_signal_watcher,
        download_signal_watcher,
        processing_list_watcher,
    };

    let abort_services = {
        let media_watcher_join_handler = {
            let handle = watch_media_items(
                args.media_dir.clone(),
                media_signal_receiver.updater,
                media_signal_receiver.signal_receiver,
            )
            .await;

            shared_state
                .media_signal_watcher
                .signal_sender
                .send(())
                .await
                .expect("Update request listener was dropped. Is media watcher loop alive?");

            handle
        };

        let bittorrent_client_join_handle = {
            let client = QBittorrentClient::try_new(Some(download_path)).unwrap();

            let handle = tokio::spawn(async move {
                client
                    .event_loop(
                        download_signal_receiver.signal_receiver,
                        download_signal_receiver.updater,
                    )
                    .await
                    .expect("Event loop exited sooner than expected");
            });

            let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
            shared_state
                .download_signal_watcher
                .signal_sender
                .send(QBittorrentClientMessage::UpdateTorrentList { result_sender })
                .await
                .unwrap();

            result_receiver.await.unwrap().unwrap();

            handle
        };

        let torrent_watcher_handle = {
            let media_dir_clone = args.media_dir.clone();

            tokio::spawn(watch_and_process_downloads(
                media_dir_clone.clone(),
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
