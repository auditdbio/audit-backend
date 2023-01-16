use std::collections::HashMap;

use common::entities::role::Role;
use mongodb::{Collection, Client, error::Result, bson::{oid::ObjectId, doc, Document}};
use serde::{Serialize, Deserialize};
use futures::stream::{StreamExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditRequestModel {
    pub id: ObjectId,
    pub opener: Role,
    pub auditor_id: ObjectId,
    pub customer_id: ObjectId,
    pub project_id: ObjectId,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub comment: Option<String>,
    pub price: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditRequestRepo {
    inner: Collection<AuditRequestModel>,
}

impl AuditRequestRepo {
    const DATABASE: &'static str = "Audits";
    const COLLECTION: &'static str = "Requests";

    
    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<AuditRequestModel> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, req: &AuditRequestModel) -> Result<()> {
        self.inner.insert_one(req, None).await?;
        Ok(())
    }

    async fn requests_by(&self, document: Document) -> Result<Vec<AuditRequestModel>> {
        let cursor = self.inner.find(document, None).await?;
        let result = cursor
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        Ok(result) 
    }

    pub async fn find_by_customer(&self, id: ObjectId) -> Result<Vec<AuditRequestModel>> {
        self.requests_by(doc! {"customer_id": id}).await
    }
    
    pub async fn find_by_auditor(&self, id: ObjectId) -> Result<Vec<AuditRequestModel>> {
        self.requests_by(doc! {"auditor_id": id}).await
    }

    pub async fn find_by_project(&self, project_id: &str) -> Result<Vec<AuditRequestModel>> {
        self.requests_by(doc! {"project_id": project_id}).await
    }
    
    pub async fn find(&self, id: &ObjectId) -> Result<Option<AuditRequestModel>> {
        let res = self.inner.find_one(doc! {"id": id}, None).await?;
        Ok(res)
    }

    pub async fn update(&self, new_audit: &AuditRequestModel) -> Result<()> {
        let _old_audit = self.delete(&new_audit.id).await?.unwrap();
        self.create(new_audit).await?;
        Ok(())
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<Option<AuditRequestModel>> {
        Ok(self.inner.find_one_and_delete(doc! {"id": id}, None).await?)
    }
}