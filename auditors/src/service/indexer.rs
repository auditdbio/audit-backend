use anyhow::bail;
use common::{
    access_rules::{AccessRules, GetData},
    context::Context,
    entities::auditor::Auditor,
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
            bail!("No access to get auditor data")
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
}
