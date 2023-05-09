use anyhow::bail;
use common::{
    access_rules::{AccessRules, GetData},
    context::Context,
    entities::auditor::{Auditor, PublicAuditor},
};
use mongodb::bson::{oid::ObjectId, Document};

pub struct IndexerService {
    context: Context,
}

impl IndexerService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn index_auditor(&self, since: i64) -> anyhow::Result<Vec<Document>> {
        let auth = self.context.auth();

        if !GetData::get_access(auth, ()) {
            bail!("No access to get auditor data {:?}", auth)
        }

        let Some(customers) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let customers = customers.get_all_since(since).await?;

        Ok(customers
            .into_iter()
            .filter_map(|x| x.into())
            .collect::<Vec<_>>())
    }

    pub async fn find_auditors(&self, ids: Vec<ObjectId>) -> anyhow::Result<Vec<PublicAuditor>> {
        let auth = self.context.auth();

        if !GetData::get_access(auth, ()) {
            bail!("No access to get auditor data: {:?}", auth)
        }

        let Some(auditors) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let auditors = auditors.find_all_by_ids("user_id", ids).await?;

        Ok(auditors.into_iter().map(|x| x.into()).collect::<Vec<_>>())
    }
}
