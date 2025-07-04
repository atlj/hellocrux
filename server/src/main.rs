use std::sync::Arc;

use axum::{Json, Router, ServiceExt, extract, routing::get};
use clap::Parser;
use domain::Media;
use log::info;
use server::{Args, media::get_media_items};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

struct AppState {
    args: Args,
    entries: Vec<Media>,
}

type State = extract::State<Arc<AppState>>;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    info!("Crawling media items");

    let entries = get_media_items(args.media_dir.clone()).await;

    info!("Found {:#?} media items", entries.len());

    let shared_state = Arc::new(AppState {
        args: args.clone(),
        entries,
    });

    let app = Router::new()
        .nest_service("/static", ServeDir::new(args.media_dir))
        .route("/health", get(health_handler))
        .route("/get_movies", get(movie_list_handler))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Server can't be started");

    info!("Starting the server");

    axum::serve(listener, app).await.unwrap();
}

async fn movie_list_handler(extract::State(state): State) -> Json<Vec<Media>> {
    Json(state.entries.clone())
}

async fn health_handler() -> String {
    "alive".to_string()
}
