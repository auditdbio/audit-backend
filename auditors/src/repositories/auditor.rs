use common::{entities::auditor::Auditor, repository::TaggableEntityRepository};
use mongodb::bson::{oid::ObjectId, Bson};

use std::sync::Arc;

#[derive(Clone)]
pub struct AuditorRepo(
    Arc<dyn TaggableEntityRepository<Auditor, Error = mongodb::error::Error> + Send + Sync>,
);

impl AuditorRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: TaggableEntityRepository<Auditor, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(&self, user: &Auditor) -> Result<bool, mongodb::error::Error> {
        self.0.create(user).await
    }

    pub async fn find(&self, id: ObjectId) -> Result<Option<Auditor>, mongodb::error::Error> {
        self.0.find("user_id", &Bson::ObjectId(id)).await
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<Option<Auditor>, mongodb::error::Error> {
        self.0.delete(id).await
    }

    pub async fn find_all(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<Auditor>, mongodb::error::Error> {
        self.0.find_all(skip, limit).await
    }

    pub async fn find_by_tags(
        &self,
        tags: Vec<String>,
    ) -> Result<Vec<Auditor>, mongodb::error::Error> {
        self.0.find_by_tags(tags).await
    }
}
