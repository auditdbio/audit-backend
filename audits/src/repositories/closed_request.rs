use std::sync::Arc;

use common::{entities::audit_request::AuditRequest, repository::Repository};
use mongodb::error::Result;

#[derive(Clone)]
pub struct ClosedAuditRequestRepo(
    Arc<dyn Repository<AuditRequest, Error = mongodb::error::Error> + Send + Sync>,
);

impl ClosedAuditRequestRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<AuditRequest, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(&self, request: &AuditRequest) -> Result<()> {
        self.0.create(request).await.map(|_| ())
    }
}
