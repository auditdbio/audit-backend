use std::collections::HashMap;

use common::entities::auditor::Auditor;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    error::Result as MongoResult,
    Client, Collection,
};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct AuditorRepository {
    inner: Collection<Auditor>,
}

impl AuditorRepository {
    const DATABASE: &'static str = "Auditors";
    const COLLECTION: &'static str = "Auditors";

    #[allow(dead_code)] // is says that this function is not used, but it is used in main.rs
    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<Auditor> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, auditor: &Auditor) -> Result<bool> {
        let exited_auditor = self.find(&auditor.user_id).await?;

        if exited_auditor.is_some() {
            return Ok(false);
        }

        self.inner.insert_one(auditor, None).await?;
        Ok(true)
    }

    pub async fn find(&self, user_id: &ObjectId) -> Result<Option<Auditor>> {
        Ok(self.inner.find_one(doc! {"user_id": user_id}, None).await?)
    }

    pub async fn delete(&self, user_id: ObjectId) -> Result<Option<Auditor>> {
        Ok(self
            .inner
            .find_one_and_delete(doc! {"user_id": user_id}, None)
            .await?)
    }

    pub async fn request_with_tags(&self, tags: Vec<String>) -> Result<Vec<Auditor>> {
        let filter = doc! {
            "tags": doc!{
                "$elemMatch": doc!{"$in": tags}
            }
        };

        let result: Vec<Auditor> = self
            .inner
            .find(filter, None)
            .await?
            .collect::<Vec<MongoResult<Auditor>>>()
            .await
            .into_iter()
            .collect::<MongoResult<_>>()?;
        Ok(result)
    }
}
