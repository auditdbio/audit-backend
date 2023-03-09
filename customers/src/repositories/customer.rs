use common::{entities::customer::Customer, repository::Repository};
use mongodb::bson::{oid::ObjectId, Bson};

use std::sync::Arc;

#[derive(Clone)]
pub struct CustomerRepo(Arc<dyn Repository<Customer<ObjectId>, Error = mongodb::error::Error> + Send + Sync>);

impl CustomerRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<Customer<ObjectId>, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(&self, user: &Customer<ObjectId>) -> Result<bool, mongodb::error::Error> {
        self.0.create(user).await
    }

    pub async fn find(&self, id: ObjectId) -> Result<Option<Customer<ObjectId>>, mongodb::error::Error> {
        self.0.find("user_id", &Bson::ObjectId(id)).await
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<Option<Customer<ObjectId>>, mongodb::error::Error> {
        self.0.delete("user_id", id).await
    }

    pub async fn find_all(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<Customer<ObjectId>>, mongodb::error::Error> {
        self.0.find_all(skip, limit).await
    }
}
