use common::entities::audit::Audit;
use mongodb::{error::Result, Client, Collection};

#[derive(Debug, Clone)]
pub struct ClosedAuditRepo {
    inner: Collection<Audit>,
}

impl ClosedAuditRepo {
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
