use common::entities::audit_request::AuditRequest;
use mongodb::{error::Result, Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClosedRequestModel {
    pub request: AuditRequest,
    pub declined: bool,
}

#[derive(Debug, Clone)]
pub struct ClosedRequestRepo {
    inner: Collection<ClosedRequestModel>,
}

impl ClosedRequestRepo {
    const DATABASE: &'static str = "Audits";
    const COLLECTION: &'static str = "ClosedRequests";

    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<ClosedRequestModel> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, req: ClosedRequestModel) -> Result<()> {
        self.inner.insert_one(req, None).await?;
        Ok(())
    }
}
