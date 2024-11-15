use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSchema {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub genre: String,
    pub price: f64,
    pub release_year: i32,
    pub publisher: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameSchema {
    pub title: String,
    pub genre: String,
    pub price: f64,
    pub release_year: i32,
    pub publisher: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGameSchema {
    pub title: Option<String>,
    pub genre: Option<String>,
    pub price: Option<f64>,
    pub release_year: Option<i32>,
    pub publisher: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameAggregateResult {
    #[serde(rename = "_id")]
    pub genre: String,
    pub avg_price: f64,
    pub total_games: i32,
} 