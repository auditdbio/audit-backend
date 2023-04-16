pub mod http_repository;
pub mod mongo_repository;
pub mod test_repository;

use std::sync::Arc;

use async_trait::async_trait;
use mongodb::bson::{oid::ObjectId, Bson};

pub trait Entity {
    fn id(&self) -> ObjectId;
    fn timestamp(&self) -> i64;
}

#[async_trait]
pub trait Repository<T> {
    async fn insert(&self, item: &T) -> anyhow::Result<bool>;
    async fn find(&self, field: &str, value: &Bson) -> anyhow::Result<Option<T>>;
    async fn delete(&self, field: &str, item: &ObjectId) -> anyhow::Result<Option<T>>;
    async fn find_many(&self, field: &str, value: &Bson) -> anyhow::Result<Vec<T>>;
    async fn find_all(&self, skip: u32, limit: u32) -> anyhow::Result<Vec<T>>;
    async fn get_all_since(&self, since: i64) -> anyhow::Result<Vec<T>>;
}

pub type RepositoryObject<T> = Arc<dyn Repository<T>>;
