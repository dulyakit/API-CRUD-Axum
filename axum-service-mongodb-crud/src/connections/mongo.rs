use mongodb::{Client, Database};
use std::sync::Arc;

#[derive(Clone)]
pub struct MongoDb {
    pub database: Database,
    client: Client,
}

impl MongoDb {
    pub async fn connect() -> Arc<Self> {
        let mongo_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
        let database_name = std::env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");
        
        let client = Client::with_uri_str(mongo_uri)
            .await
            .expect("Failed to connect to MongoDB");
        
        let database = client.database(&database_name);
        
        println!("Connected to MongoDB successfully");
        
        Arc::new(MongoDb { 
            database,
            client,
        })
    }

    pub async fn disconnect(&self) {
        self.client.clone().shutdown().await;
        println!("Disconnected from MongoDB");
    }
}

impl Drop for MongoDb {
    fn drop(&mut self) {
        println!("Cleaning up MongoDB connection...");
    }
} 