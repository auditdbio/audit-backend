use common::{entities::audit::Audit, repository::Repository};

use mongodb::bson::{oid::ObjectId, Bson};

use std::sync::Arc;

#[derive(Clone)]
pub struct AuditRepo(Arc<dyn Repository<Audit, Error = mongodb::error::Error> + Send + Sync>);

impl AuditRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<Audit, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(&self, user: &Audit) -> Result<bool, mongodb::error::Error> {
        self.0.create(user).await
    }

    pub async fn find(&self, id: ObjectId) -> Result<Option<Audit>, mongodb::error::Error> {
        self.0.find("id", &Bson::ObjectId(id)).await
    }

    pub async fn find_by_customer(
        &self,
        customer_id: ObjectId,
    ) -> Result<Vec<Audit>, mongodb::error::Error> {
        self.0.find_many("id", &Bson::ObjectId(customer_id)).await
    }

    pub async fn find_by_auditor(
        &self,
        auditor_id: ObjectId,
    ) -> Result<Vec<Audit>, mongodb::error::Error> {
        self.0.find_many("id", &Bson::ObjectId(auditor_id)).await
    }

    pub async fn find_by_project(
        &self,
        project_id: ObjectId,
    ) -> Result<Vec<Audit>, mongodb::error::Error> {
        self.0.find_many("id", &Bson::ObjectId(project_id)).await
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<Option<Audit>, mongodb::error::Error> {
        self.0.delete("id", id).await
    }

    pub async fn find_all(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<Audit>, mongodb::error::Error> {
        self.0.find_all(skip, limit).await
    }
}
