use common::{
    access_rules::{AccessRules, GetData},
    context::Context,
    entities::auditor::{Auditor, PublicAuditor},
    error::{self, AddCode},
};
use mongodb::bson::{oid::ObjectId, Document};

pub struct IndexerService {
    context: Context,
}

impl IndexerService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn index_auditor(&self, since: i64) -> error::Result<Vec<Document>> {
        let auth = self.context.auth();

        if !GetData.get_access(auth, ()) {
            return Err(anyhow::anyhow!("No access to get auditor data {:?}", auth).code(400));
        }

        let customers = self.context.try_get_repository::<Auditor<ObjectId>>()?;

        let customers = customers.get_all_since(since).await?;

        Ok(customers
            .into_iter()
            .filter_map(|x| x.into())
            .collect::<Vec<_>>())
    }

    pub async fn find_auditors(&self, ids: Vec<ObjectId>) -> error::Result<Vec<PublicAuditor>> {
        let auth = self.context.auth();

        if !GetData.get_access(auth, ()) {
            return Err(anyhow::anyhow!("No access to get auditor data: {:?}", auth).code(400));
        }

        let auditors = self.context.try_get_repository::<Auditor<ObjectId>>()?;

        let auditors = auditors.find_all_by_ids("user_id", ids).await?;

        Ok(auditors
            .into_iter()
            .map(|x| auth.public_auditor(x))
            .collect::<Vec<_>>())
    }
}
