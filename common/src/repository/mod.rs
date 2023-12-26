pub mod http_repository;
pub mod mongo_repository;
pub mod test_repository;

use std::sync::Arc;

use async_trait::async_trait;
use mongodb::bson::{oid::ObjectId, Bson};

use crate::error;

pub trait Entity {
    fn id(&self) -> ObjectId;
}

#[async_trait]
pub trait Repository<T> {
    async fn insert(&self, item: &T) -> error::Result<bool>;
    async fn find(&self, field: &str, value: &Bson) -> error::Result<Option<T>>;
    async fn delete(&self, field: &str, item: &ObjectId) -> error::Result<Option<T>>;
    async fn find_many(&self, field: &str, value: &Bson) -> error::Result<Vec<T>>;
    async fn find_many_limit(&self, field: &str, value: &Bson, skip: i32, limit: i32) -> error::Result<(Vec<T>, u64)>;
    async fn find_all(&self, skip: u32, limit: u32) -> error::Result<Vec<T>>;
    async fn get_all_since(&self, since: i64) -> error::Result<Vec<T>>;
    async fn find_all_by_ids(&self, id: &str, ids: Vec<ObjectId>) -> error::Result<Vec<T>>;
}

pub type RepositoryObject<T> = Arc<dyn Repository<T> + Send + Sync + 'static>;
