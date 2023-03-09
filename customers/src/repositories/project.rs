use common::{entities::project::Project, repository::TaggableEntityRepository};

use mongodb::bson::{oid::ObjectId, Bson};

use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectRepo(
    Arc<dyn TaggableEntityRepository<Project<ObjectId>, Error = mongodb::error::Error> + Send + Sync>,
);

impl ProjectRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: TaggableEntityRepository<Project<ObjectId>, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(&self, user: &Project<ObjectId>) -> Result<bool, mongodb::error::Error> {
        self.0.create(user).await
    }

    pub async fn find_by_customer(
        &self,
        customer_id: ObjectId,
    ) -> Result<Vec<Project<ObjectId>>, mongodb::error::Error> {
        self.0.find_many("customer_id", &Bson::ObjectId(customer_id)).await
    }

    pub async fn find(&self, id: ObjectId) -> Result<Option<Project<ObjectId>>, mongodb::error::Error> {
        self.0.find("id", &Bson::ObjectId(id)).await
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<Option<Project<ObjectId>>, mongodb::error::Error> {
        self.0.delete("id", id).await
    }

    pub async fn find_all(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<Project<ObjectId>>, mongodb::error::Error> {
        self.0.find_all(skip, limit).await
    }

    pub async fn find_by_tags(
        &self,
        tags: Vec<String>,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<Project<ObjectId>>, mongodb::error::Error> {
        self.0.find_by_tags(tags, skip, limit).await
    }
}
