use mongodb::{Collection, Client, error::Result, bson::{oid::ObjectId, doc, Document}};
use serde::{Serialize, Deserialize};
use futures::stream::{StreamExt, TryStreamExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditRequestModel {
    pub id: ObjectId,
    pub opener: Role,
    pub auditor_email: String,
    pub customer_email: String,
    pub project_id: String,
    pub auditor_contacts: Vec<String>,
    pub customer_contacts: Vec<String>,
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

    pub async fn requests_by_customer(&self, email: &str) -> Result<Vec<AuditRequestModel>> {
        self.requests_by(doc! {"customer_email": email}).await
    }
    
    pub async fn requests_by_auditor(&self, email: &str) -> Result<Vec<AuditRequestModel>> {
        self.requests_by(doc! {"auditor_email": email}).await
    }

    pub async fn requests_by_project(&self, project_id: &str) -> Result<Vec<AuditRequestModel>> {
        self.requests_by(doc! {"project_id": project_id}).await
    }
    
    pub async fn request(&self, uuid: &str) -> Result<AuditRequestModel> {
        let res = self.inner.find_one(doc! {"uuid": uuid}, None).await?;
        Ok(res.unwrap())
    }

    pub async fn update(&self, new_audit: &AuditRequestModel) -> Result<()> {
        let _old_audit = self.remove(&new_audit.uuid).await?.unwrap();
        self.create(new_audit).await?;
        Ok(())
    }

    pub async fn remove(&self, uuid: &str) -> Result<Option<AuditRequestModel>> {
        Ok(self.inner.find_one_and_delete(doc! {"uuid": uuid}, None).await?)
    }
}