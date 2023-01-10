use futures::StreamExt;
use mongodb::{Collection, Client, error::Result, bson::{doc, DateTime, oid::ObjectId, Bson}};
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct AuditModel {
    id: Option<ObjectId>,
    scope: Scope,
    terms: String,
    status: String,
    last_modified: DateTime,
    issues: Vec<Issue>,
    visibility: Vec<String>,
    project_id: String,
    auditor_email: String,
}

#[derive(Debug, Clone)]
pub struct AuditRepo {
    inner: Collection<AuditModel>,
}

impl AuditRepo {
    const DATABASE: &'static str = "Audits";
    const COLLECTION: &'static str = "Audits";

    
    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<AuditModel> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, mut audit: AuditModel) -> Result<AuditModel> {
        let res = self.inner.insert_one(&audit, None).await?;
        if let Bson::ObjectId(id) = res.inserted_id {
            audit.id = Some(id);
            return  Ok(audit);
        }
        unreachable!()
    }

    pub async fn get_audits(&self, user_email: &str) -> Result<Vec<AuditModel>> {
        let cursor = self.inner.find(doc!{"visibility": doc!{ "$matchElem": {"$eq": user_email}}}, None).await?;
        let result = cursor
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        Ok(result) 
    }

    pub async fn update_audit(&self, new_audit: &AuditModel) -> Result<()> {
        todo!()
    }

    pub async fn remove_audit(&self, audit_id: ObjectId) -> Result<Option<AuditModel>> {
        Ok(self.inner.find_one_and_delete(doc!{"id": audit_id}, None).await?)
    }
}
