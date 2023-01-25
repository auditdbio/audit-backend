use std::collections::HashMap;

use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    error::Result as MongoResult,
    Client, Collection,
};
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditorModel {
    pub user_id: ObjectId,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub tags: Vec<String>,
    pub contacts: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct AuditorRepository {
    inner: Collection<AuditorModel>,
}

impl AuditorRepository {
    const DATABASE: &'static str = "Auditors";
    const COLLECTION: &'static str = "Auditors";

    #[allow(dead_code)] // is says that this function is not used, but it is used in main.rs
    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<AuditorModel> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, auditor: &AuditorModel) -> Result<bool> {
        let exited_auditor = self.find(&auditor.user_id).await?;

        if exited_auditor.is_some() {
            return Ok(false);
        }

        self.inner.insert_one(auditor, None).await?;
        Ok(true)
    }

    pub async fn find(&self, user_id: &ObjectId) -> Result<Option<AuditorModel>> {
        Ok(self.inner.find_one(doc! {"user_id": user_id}, None).await?)
    }

    pub async fn delete(&self, user_id: ObjectId) -> Result<Option<AuditorModel>> {
        Ok(self
            .inner
            .find_one_and_delete(doc! {"user_id": user_id}, None)
            .await?)
    }

    pub async fn request_with_tags(&self, tags: Vec<String>) -> Result<Vec<AuditorModel>> {
        let filter = doc! {
            "tags": doc!{
                "$elemMatch": doc!{"$in": tags}
            }
        };

        let result: Vec<AuditorModel> = self
            .inner
            .find(filter, None)
            .await?
            .collect::<Vec<MongoResult<AuditorModel>>>()
            .await
            .into_iter()
            .collect::<MongoResult<_>>()?;
        Ok(result)
    }
}
