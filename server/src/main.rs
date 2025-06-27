use axum::{Json, Router, routing::get};
use domain::Media;
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

async fn movie_list_handler() -> Json<Vec<Media>> {
    Json(
        vec![
        Media {
            id:"1".to_string(),
            thumbnail: "https://m.media-amazon.com/images/M/MV5BMTkzMzM3OTM2Ml5BMl5BanBnXkFtZTgwMDM0NDU3MjI@._V1_FMjpg_UY2048_.jpg".to_string(),
            title: "Emoji Movie".to_string()
        },
        Media {
            id: "2".to_string(),
            thumbnail: "https://www.themoviedb.org/t/p/w600_and_h900_bestv2/unPB1iyEeTBcKiLg8W083rlViFH.jpg".to_string(),
            title: "The Boss Baby".to_string()
        },
        Media {
            id: "3".to_string(),
            thumbnail: "https://www.themoviedb.org/t/p/w600_and_h900_bestv2/78lPtwv72eTNqFW9COBYI0dWDJa.jpg".to_string(),
            title: "Iron Man".to_string()
        },
        Media {
            id: "4".to_string(),
            thumbnail: "https://www.themoviedb.org/t/p/w600_and_h900_bestv2/9cqNxx0GxF0bflZmeSMuL5tnGzr.jpg".to_string(),
            title: "Shawshank Redemption".to_string()
        },
        ]
        )
}

async fn health_handler() -> String {
    "alive".to_string()
}
