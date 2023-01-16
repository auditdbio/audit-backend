use common::entities::audit::Audit;
use futures::StreamExt;
use mongodb::{Collection, Client, error::Result, bson::{doc, oid::ObjectId}};

#[derive(Debug, Clone)]
pub struct AuditRepo {
    inner: Collection<Audit>,
}

impl AuditRepo {
    const DATABASE: &'static str = "Audits";
    const COLLECTION: &'static str = "Audits";

    
    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<Audit> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, audit: &Audit) -> Result<()> {
        self.inner.insert_one(audit, None).await?;
        Ok(())
    }

    pub async fn find(&self, audit_id: ObjectId) -> Result<Option<Audit>> {
        Ok(self.inner.find_one(doc!{"id": audit_id}, None).await?)
    }

    pub async fn get_audits(&self, user_id: &ObjectId) -> Result<Vec<Audit>> {
        let cursor = self.inner.find(doc!{"visibility": doc!{ "$matchElem": {"$eq": user_id}}}, None).await?;
        let result = cursor
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        Ok(result) 
    }

    pub async fn delete(&self, audit_id: ObjectId) -> Result<Option<Audit>> {
        Ok(self.inner.find_one_and_delete(doc!{"id": audit_id}, None).await?)
    }
}
