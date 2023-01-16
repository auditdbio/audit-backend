use common::entities::audit::Audit;
use futures::StreamExt;
use mongodb::{Collection, Client, error::Result, bson::{doc, DateTime, oid::ObjectId, Bson}};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct AuditRepo {
    inner: Collection<Audit>,
}

impl AuditRepo {
    const DATABASE: &'static str = "Audits";
    const COLLECTION: &'static str = "ClosedAudits";

    
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
}
