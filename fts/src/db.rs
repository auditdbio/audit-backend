use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{ClientOptions, FindOptions},
    Client, Collection, Database,
};

use crate::{
    error::{ServiceError, ServiceResult},
    models::Auditor,
};

pub struct MongoDb {
    collection: Collection<Auditor>,
}

impl MongoDb {
    pub async fn new(uri: &str, db_name: &str) -> ServiceResult<Self> {
        let mut client_options = ClientOptions::parse(uri).await?;
        client_options.app_name = Some("fts-service".to_string());

        let client = Client::with_options(client_options)?;
        let db: Database = client.database(db_name);
        let collection = db.collection("auditors");

        Ok(Self { collection })
    }

    pub async fn get_auditors_since(&self, timestamp: i64) -> ServiceResult<Vec<Auditor>> {
        let filter = doc! {
            "last_modified": { "$gt": timestamp }
        };

        let mut cursor = self.collection.find(filter, None).await?;
        let mut auditors = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(auditor) => auditors.push(auditor),
                Err(e) => tracing::error!("Error fetching auditor: {}", e),
            }
        }

        Ok(auditors)
    }

    pub async fn get_auditors_by_ids(&self, ids: &[String]) -> ServiceResult<Vec<Auditor>> {
        let object_ids: Vec<ObjectId> = ids
            .iter()
            .filter_map(|id| ObjectId::parse_str(id).ok())
            .collect();

        let filter = doc! {
            "_id": { "$in": object_ids }
        };

        let options = FindOptions::builder()
            .sort(doc! { "last_modified": -1 })
            .build();

        let mut cursor = self.collection.find(filter, options).await?;
        let mut auditors = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(auditor) => auditors.push(auditor),
                Err(e) => tracing::error!("Error fetching auditor: {}", e),
            }
        }

        Ok(auditors)
    }

    pub async fn check_auditor_exists(&self, id: &str) -> ServiceResult<bool> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|e| ServiceError::Query(format!("Invalid ObjectId: {}", e)))?;

        let filter = doc! {
            "_id": object_id
        };

        Ok(self.collection.count_documents(filter, None).await? > 0)
    }
} 