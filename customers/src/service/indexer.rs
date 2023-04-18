use anyhow::bail;
use common::{
    access_rules::{AccessRules, GetData},
    context::Context,
    entities::{customer::Customer, project::Project},
};
use mongodb::bson::{oid::ObjectId, Document};

pub struct IndexerService {
    context: Context,
}

impl IndexerService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn index_customer(&self, since: i64) -> anyhow::Result<Vec<Document>> {
        let auth = self.context.auth();

        if !GetData::get_access(auth, ()) {
            bail!("No access to get customer data")
        }

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let customers = customers.get_all_since(since).await?;

        Ok(customers
            .into_iter()
            .filter_map(|x| x.into())
            .collect::<Vec<_>>())
    }

    pub async fn index_project(&self, since: i64) -> anyhow::Result<Vec<Document>> {
        let auth = self.context.auth();

        if !GetData::get_access(auth, ()) {
            bail!("No access to get customer data")
        }
        let Some(customers) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let customers = customers.get_all_since(since).await?;

        Ok(customers
            .into_iter()
            .filter_map(|x| x.into())
            .collect::<Vec<_>>())
    }
}
