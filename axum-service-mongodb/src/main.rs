mod connections;
mod models;

use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Router,
    response::Json,
    http::StatusCode,
};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
};
use futures::TryStreamExt;
use std::sync::Arc;
use dotenv::dotenv;
use connections::mongo::MongoDb;
use models::game::{GameSchema, CreateGameSchema, UpdateGameSchema, GameAggregateResult};

// Create game
async fn create_game(
    State(mongo): State<Arc<MongoDb>>,
    Json(game): Json<CreateGameSchema>,
) -> Result<Json<GameSchema>, StatusCode> {
    let collection = mongo.database.collection::<GameSchema>("games");
    
    let game = GameSchema {
        id: None,
        title: game.title,
        genre: game.genre,
        price: game.price,
        release_year: game.release_year,
        publisher: game.publisher,
    };

    let result = collection
        .insert_one(game, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let created_game = collection
        .find_one(doc! {"_id": result.inserted_id}, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(created_game))
}

// Get all games
async fn get_games(
    State(mongo): State<Arc<MongoDb>>,
) -> Result<Json<Vec<GameSchema>>, StatusCode> {
    let collection = mongo.database.collection::<GameSchema>("games");
    
    let mut games = Vec::new();
    let mut cursor = collection
        .find(None, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(game) = cursor
        .try_next()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        games.push(game);
    }

    Ok(Json(games))
}

// Get game stats by genre
async fn get_genre_stats(
    State(mongo): State<Arc<MongoDb>>,
) -> Result<Json<Vec<GameAggregateResult>>, StatusCode> {
    let collection = mongo.database.collection::<GameSchema>("games");
    
    let pipeline = vec![
        doc! {
            "$group": {
                "_id": "$genre",
                "avg_price": {"$avg": "$price"},
                "total_games": {"$sum": 1}
            }
        },
        doc! {
            "$sort": {"total_games": -1}
        }
    ];

    let mut results = Vec::new();
    let mut cursor = collection
        .aggregate(pipeline, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(result) = cursor
        .try_next()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let genre_stat: GameAggregateResult = mongodb::bson::from_document(result)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        results.push(genre_stat);
    }

    Ok(Json(results))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mongo = MongoDb::connect().await;

    let app = Router::new()
        .route("/games", post(create_game))
        .route("/games", get(get_games))
        .route("/stats/genres", get(get_genre_stats))
        .with_state(mongo.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    
    let shutdown_signal = async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        println!("\nReceived shutdown signal...");
        mongo.disconnect().await;
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();
} 