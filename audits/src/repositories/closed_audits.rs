use std::sync::Arc;

use common::{entities::audit::Audit, repository::Repository};
use mongodb::{bson::oid::ObjectId, error::Result};

#[derive(Clone)]
pub struct ClosedAuditRepo(
    Arc<dyn Repository<Audit<ObjectId>, Error = mongodb::error::Error> + Send + Sync>,
);

impl ClosedAuditRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<Audit<ObjectId>, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(&self, audit: &Audit<ObjectId>) -> Result<()> {
        self.0.create(audit).await.map(|_| ())
    }

    pub async fn get_all_since(&self, since: i64) -> Result<Vec<Audit<ObjectId>>> {
        self.0.get_all_since(since).await
    }
}
