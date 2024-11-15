use mongodb::bson::{oid::ObjectId, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSchema {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub city: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserSchema {
    pub name: String,
    pub email: String,
    pub age: i32,
    pub city: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserSchema {
    pub name: Option<String>,
    pub email: Option<String>,
    pub age: Option<i32>,
    pub city: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAggregateResult {
    pub _id: String,  // city
    pub avg_age: f64,
    pub total_users: i32,
} 