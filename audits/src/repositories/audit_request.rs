use common::{entities::audit_request::AuditRequest, repository::Repository};

use mongodb::bson::{oid::ObjectId, Bson};

use std::sync::Arc;

#[derive(Clone)]
pub struct AuditRequestRepo(
    Arc<dyn Repository<AuditRequest<ObjectId>, Error = mongodb::error::Error> + Send + Sync>,
);

impl AuditRequestRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<AuditRequest<ObjectId>, Error = mongodb::error::Error>
            + Send
            + Sync
            + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(
        &self,
        user: &AuditRequest<ObjectId>,
    ) -> Result<bool, mongodb::error::Error> {
        self.0.create(user).await
    }

    pub async fn find(
        &self,
        id: ObjectId,
    ) -> Result<Option<AuditRequest<ObjectId>>, mongodb::error::Error> {
        self.0.find("id", &Bson::ObjectId(id)).await
    }

    pub async fn find_by_customer(
        &self,
        customer_id: ObjectId,
    ) -> Result<Vec<AuditRequest<ObjectId>>, mongodb::error::Error> {
        self.0.find_many("id", &Bson::ObjectId(customer_id)).await
    }

    pub async fn find_by_auditor(
        &self,
        auditor_id: ObjectId,
    ) -> Result<Vec<AuditRequest<ObjectId>>, mongodb::error::Error> {
        self.0.find_many("id", &Bson::ObjectId(auditor_id)).await
    }

    pub async fn find_by_project(
        &self,
        project_id: ObjectId,
    ) -> Result<Vec<AuditRequest<ObjectId>>, mongodb::error::Error> {
        self.0.find_many("id", &Bson::ObjectId(project_id)).await
    }

    pub async fn delete(
        &self,
        id: &ObjectId,
    ) -> Result<Option<AuditRequest<ObjectId>>, mongodb::error::Error> {
        self.0.delete("id", id).await
    }

    pub async fn find_all(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<AuditRequest<ObjectId>>, mongodb::error::Error> {
        self.0.find_all(skip, limit).await
    }
}
