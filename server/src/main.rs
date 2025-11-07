use std::sync::Arc;

use axum::{
    Json, Router, extract,
    routing::{get, post},
};
use clap::Parser;
use domain::Media;
use log::info;
use server::{Args, media::get_media_items};
use tokio::net::TcpListener;
use torrent::{TorrentInfo, qbittorrent_client::QBittorrentClientMessage};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    entries: Arc<[Media]>,
    download_channels: (
        tokio::sync::mpsc::Sender<QBittorrentClientMessage>,
        tokio::sync::watch::Receiver<Box<[TorrentInfo]>>,
    ),
}

type State = extract::State<AppState>;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

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

    join_handle.abort();
}

async fn movie_list_handler(extract::State(state): State) -> Json<Box<[Media]>> {
    Json(state.entries.as_ref().into())
}

async fn health_handler() -> String {
    "alive".to_string()
}

mod download_handlers {
    use std::path::PathBuf;

    use axum::{Json, extract, http::StatusCode};
    use domain::Download;
    use tokio::task::JoinHandle;
    use torrent::{
        TorrentInfo,
        qbittorrent_client::{QBittorrentClient, QBittorrentClientMessage},
    };

    use crate::State;

    pub async fn spawn_download_event_loop(
        path: PathBuf,
    ) -> (
        tokio::sync::mpsc::Sender<QBittorrentClientMessage>,
        tokio::sync::watch::Receiver<Box<[TorrentInfo]>>,
        JoinHandle<()>,
    ) {
        let client = QBittorrentClient::try_new(Some(path)).unwrap();
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let (list_sender, list_receiver) =
            tokio::sync::watch::channel::<Box<[TorrentInfo]>>(Box::new([]));

        let handle = tokio::spawn(async move {
            client
                .event_loop(receiver, list_sender)
                .await
                .expect("Event loop exited sooner than expected");
        });

        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
        sender
            .send(QBittorrentClientMessage::UpdateTorrentList { result_sender })
            .await
            .unwrap();

        result_receiver.await.unwrap().unwrap();

        (sender, list_receiver, handle)
    }

    pub async fn get_downloads(extract::State(state): State) -> Json<Box<[Download]>> {
        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

        // TODO: make this a periodic call.
        state
            .download_channels
            .0
            .send(QBittorrentClientMessage::UpdateTorrentList { result_sender })
            .await
            .unwrap();

        result_receiver.await.unwrap().unwrap();

        Json(
            state
                .download_channels
                .1
                .borrow()
                .iter()
                .map(|torrent| torrent.clone().into())
                .collect(),
        )
    }

    pub async fn add_download(
        extract::State(state): State,
        body: String,
    ) -> axum::response::Result<()> {
        use QBittorrentClientMessage as Message;

        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

        state
            .download_channels
            .0
            .send(Message::AddTorrent {
                hash: body.into(),
                result_sender,
            })
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let result = result_receiver
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(())
    }

    pub async fn remove_download() -> axum::response::Result<()> {
        todo!()
    }

    pub async fn pause_download() -> axum::response::Result<()> {
        todo!()
    }

    pub async fn resume_download() -> axum::response::Result<()> {
        todo!()
    }
}
