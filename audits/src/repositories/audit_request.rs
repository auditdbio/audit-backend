use common::entities::audit_request::AuditRequest;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    error::Result,
    Client, Collection,
};

#[derive(Debug, Clone)]
pub struct AuditRequestRepo {
    inner: Collection<AuditRequest>,
}

impl AuditRequestRepo {
    const DATABASE: &'static str = "Audits";
    const COLLECTION: &'static str = "Requests";

    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<AuditRequest> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, req: &AuditRequest) -> Result<()> {
        self.inner.insert_one(req, None).await?;
        Ok(())
    }

    async fn find_by(&self, document: Document) -> Result<Vec<AuditRequest>> {
        let cursor = self.inner.find(document, None).await?;
        let result = cursor
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        Ok(result)
    }

    pub async fn find_by_customer(&self, id: &ObjectId) -> Result<Vec<AuditRequest>> {
        self.find_by(doc! {"customer_id": id}).await
    }

    pub async fn find_by_auditor(&self, id: &ObjectId) -> Result<Vec<AuditRequest>> {
        self.find_by(doc! {"auditor_id": id}).await
    }

    pub async fn find_by_project(&self, project_id: &str) -> Result<Vec<AuditRequest>> {
        self.find_by(doc! {"project_id": project_id}).await
    }

    pub async fn find(&self, id: &ObjectId) -> Result<Option<AuditRequest>> {
        let res = self.inner.find_one(doc! {"id": id}, None).await?;
        Ok(res)
    }

    pub async fn update(&self, new_audit: &AuditRequest) -> Result<()> {
        let _old_audit = self.delete(&new_audit.id).await?.unwrap();
        self.create(new_audit).await?;
        Ok(())
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<Option<AuditRequest>> {
        Ok(self
            .inner
            .find_one_and_delete(doc! {"id": id}, None)
            .await?)
    }
}
