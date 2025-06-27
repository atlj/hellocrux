use axum::{Json, Router, routing::get};
use domain::Movie;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/get_movies", get(movie_list_handler));

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Server can't be started");

    axum::serve(listener, app).await.unwrap();
}

async fn movie_list_handler() -> Json<Vec<Movie>> {
    Json(
        vec![
            Movie {
                id:"my-id".to_string(),
                thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg".to_string()
            }
        ]
        )
}

async fn health_handler() -> String {
    "alive".to_string()
}
