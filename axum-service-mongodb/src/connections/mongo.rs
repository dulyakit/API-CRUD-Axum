use mongodb::{Client, Database};
use std::env;

#[derive(Clone)]
pub struct MongoDb {
    pub database: Database,
    client: Client,
}

impl MongoDb {
    pub async fn connect() -> std::sync::Arc<Self> {
        let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
        let client = Client::with_uri_str(&uri)
            .await
            .expect("Failed to connect to MongoDB");
            
        let db_name = env::var("MONGODB_DB_NAME").expect("MONGODB_DB_NAME must be set");
        let database = client.database(&db_name);
        
        std::sync::Arc::new(Self { database, client })
    }

    pub async fn disconnect(&self) {
        self.client.clone().shutdown().await;
        println!("Disconnected from MongoDB");
    }
} 