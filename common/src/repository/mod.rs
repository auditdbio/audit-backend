pub mod mongo_repository;
pub mod test_repository;

use async_trait::async_trait;
use mongodb::bson::{oid::ObjectId, Bson};


pub trait Entity {
    fn id(&self) -> ObjectId;
}

pub trait TaggableEntity: Entity {
    fn tags(&self) -> &Vec<String>;
}

#[async_trait]
pub trait Repository<T> {
    type Error;
    async fn create(&self, item: &T) -> Result<bool, Self::Error>;
    async fn find(&self, field: &str, value: &Bson) -> Result<Option<T>, Self::Error>;
    async fn delete(&self, field: &str, item: &ObjectId) -> Result<Option<T>, Self::Error>;
    async fn find_many(&self, field: &str, value: &Bson) -> Result<Vec<T>, Self::Error>;
    async fn find_all(&self, skip: u32, limit: u32) -> Result<Vec<T>, Self::Error>;
}

#[async_trait]
pub trait TaggableEntityRepository<T>: Repository<T> {
    async fn find_by_tags(&self, tags: Vec<String>) -> Result<Vec<T>, Self::Error>;
}
