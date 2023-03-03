
use std::sync::Arc;
use chrono::NaiveDateTime;
use common::repository::{Repository, Entity};
use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: ObjectId,
    pub creator_id: ObjectId,
    pub last_modified: NaiveDateTime,
    pub content_type: String,
    pub path: String,
}

impl Entity for Metadata {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

#[derive(Clone)]
pub struct MetadataRepo(Arc<dyn Repository<Metadata, Error = mongodb::error::Error> + Send + Sync>);

impl MetadataRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<Metadata, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }
    pub async fn create(&self, user: &Metadata) -> Result<bool, mongodb::error::Error> {
        self.0.create(user).await
    }

    pub async fn find_by_path(
        &self,
        path: String,
    ) -> Result<Option<Metadata>, mongodb::error::Error> {
        self.0.find("token", &Bson::String(path)).await
    }

    pub async fn delete(&self, path: String) -> Result<Option<Metadata>, mongodb::error::Error> {
        let token = self.find_by_path(path).await?;
        if let Some(token) = token {
            self.0.delete("id", &token.id).await
        } else {
            Ok(None)
        }
    }
}
