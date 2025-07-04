use std::sync::Arc;

use axum::{Json, Router, extract, routing::get};
use clap::Parser;
use domain::{Media, MediaContent};
use server::{Args, media::get_media_items};
use tokio::net::TcpListener;

struct AppState {
    args: Args,
    entries: Vec<Media>,
}

type State = extract::State<Arc<AppState>>;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Crawling media items");

    let entries = get_media_items(args.media_dir.clone()).await;

    let shared_state = Arc::new(AppState { args, entries });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/get_movies", get(movie_list_handler))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Server can't be started");

    axum::serve(listener, app).await.unwrap();
}

async fn movie_list_handler(extract::State(state): State) -> Json<Vec<Media>> {
    Json(state.entries.clone())
}

async fn health_handler() -> String {
    "alive".to_string()
}
