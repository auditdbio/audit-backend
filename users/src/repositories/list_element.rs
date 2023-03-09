use std::sync::Arc;

use common::{repository::{Repository, Entity}};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ListElement {
    pub id: ObjectId,
    pub email: String,
}

impl Entity for ListElement {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

#[derive(Clone)]
pub struct ListElementRepository(Arc<dyn Repository<ListElement, Error = mongodb::error::Error> + Send + Sync>);

impl ListElementRepository {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<ListElement, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(&self, user: &ListElement) -> Result<bool, mongodb::error::Error> {
        self.0.create(user).await
    }

    pub async fn find(&self, id: ObjectId) -> Result<Option<ListElement>, mongodb::error::Error> {
        self.0.find("id", &Bson::ObjectId(id)).await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<ListElement>, mongodb::error::Error> {
        self.0.find("email", &Bson::String(email.to_string())).await
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<Option<ListElement>, mongodb::error::Error> {
        self.0.delete("id", id).await
    }

    pub async fn find_all(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<ListElement>, mongodb::error::Error> {
        self.0.find_all(skip, limit).await
    }
}
