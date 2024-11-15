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
    options::AggregateOptions,
};
use futures::TryStreamExt;
use std::sync::Arc;
use dotenv::dotenv;
use connections::mongo::MongoDb;
use models::user::{UserSchema, CreateUserSchema, UpdateUserSchema, UserAggregateResult};

// Create
async fn create_user(
    State(mongo): State<Arc<MongoDb>>,
    Json(user): Json<CreateUserSchema>,
) -> Result<Json<UserSchema>, StatusCode> {
    let collection = mongo.database.collection::<UserSchema>("users");
    
    let user = UserSchema {
        id: None,
        name: user.name,
        email: user.email,
        age: user.age,
        city: user.city,
    };

    let result = collection
        .insert_one(user, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let created_user = collection
        .find_one(doc! {"_id": result.inserted_id}, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(created_user))
}

// Read all
async fn get_users(
    State(mongo): State<Arc<MongoDb>>,
) -> Result<Json<Vec<UserSchema>>, StatusCode> {
    let collection = mongo.database.collection::<UserSchema>("users");
    
    let mut users = Vec::new();
    let mut cursor = collection
        .find(None, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(user) = cursor
        .try_next()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        users.push(user);
    }

    Ok(Json(users))
}

// Read one
async fn get_user(
    State(mongo): State<Arc<MongoDb>>,
    Path(id): Path<String>,
) -> Result<Json<UserSchema>, StatusCode> {
    let object_id = ObjectId::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let collection = mongo.database.collection::<UserSchema>("users");
    
    let user = collection
        .find_one(doc! {"_id": object_id}, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(user))
}

// Update
async fn update_user(
    State(mongo): State<Arc<MongoDb>>,
    Path(id): Path<String>,
    Json(update): Json<UpdateUserSchema>,
) -> Result<Json<UserSchema>, StatusCode> {
    let object_id = ObjectId::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let collection = mongo.database.collection::<UserSchema>("users");
    
    let mut update_doc = Document::new();
    
    if let Some(name) = update.name { update_doc.insert("name", name); }
    if let Some(email) = update.email { update_doc.insert("email", email); }
    if let Some(age) = update.age { update_doc.insert("age", age); }
    if let Some(city) = update.city { update_doc.insert("city", city); }
    
    let update_doc = doc! {"$set": update_doc};

    collection
        .update_one(doc! {"_id": object_id}, update_doc, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated_user = collection
        .find_one(doc! {"_id": object_id}, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(updated_user))
}

// Delete
async fn delete_user(
    State(mongo): State<Arc<MongoDb>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let object_id = ObjectId::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let collection = mongo.database.collection::<UserSchema>("users");
    
    let result = collection
        .delete_one(doc! {"_id": object_id}, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.deleted_count == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

// Aggregate - Average age by city
async fn get_city_stats(
    State(mongo): State<Arc<MongoDb>>,
) -> Result<Json<Vec<UserAggregateResult>>, StatusCode> {
    let collection = mongo.database.collection::<UserSchema>("users");
    
    let pipeline = vec![
        doc! {
            "$group": {
                "_id": "$city",
                "avg_age": {"$avg": "$age"},
                "total_users": {"$sum": 1}
            }
        },
        doc! {
            "$sort": {"total_users": -1}
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
        let city_stat: UserAggregateResult = mongodb::bson::from_document(result)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        results.push(city_stat);
    }

    Ok(Json(results))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mongo = MongoDb::connect().await;

    let app = Router::new()
        .route("/users", post(create_user))
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .route("/stats/cities", get(get_city_stats))
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