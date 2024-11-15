use axum::{
    routing::get,
    Router,
    response::Json,
};
use serde_json::{json, Value};

async fn hello_world() -> Json<Value> {
    Json(json!({ "message": "Hello, World!" }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello_world));

    // เริ่ม server ที่ port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await.unwrap();
} 