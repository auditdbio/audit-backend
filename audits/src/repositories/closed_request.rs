use std::sync::Arc;

use common::{entities::audit_request::AuditRequest, repository::Repository};
use mongodb::{bson::oid::ObjectId, error::Result};

#[derive(Clone)]
pub struct ClosedAuditRequestRepo(
    Arc<dyn Repository<AuditRequest<ObjectId>, Error = mongodb::error::Error> + Send + Sync>,
);

impl ClosedAuditRequestRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<AuditRequest<ObjectId>, Error = mongodb::error::Error>
            + Send
            + Sync
            + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(&self, request: &AuditRequest<ObjectId>) -> Result<()> {
        self.0.create(request).await.map(|_| ())
    }

    pub async fn get_all_since(&self, since: i64) -> Result<Vec<AuditRequest<ObjectId>>> {
        self.0.get_all_since(since).await
    }
}
