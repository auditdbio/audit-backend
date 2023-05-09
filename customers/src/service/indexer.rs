use anyhow::bail;
use common::{
    access_rules::{AccessRules, GetData},
    context::Context,
    entities::{
        customer::{Customer, PublicCustomer},
        project::{Project, PublicProject},
    },
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
            bail!("No access to get customer data {:?}", auth)
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
            bail!("No access to get customer data: {:?}", auth)
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

    pub async fn find_customers(&self, ids: Vec<ObjectId>) -> anyhow::Result<Vec<PublicCustomer>> {
        let auth = self.context.auth();

        if !GetData::get_access(auth, ()) {
            bail!("No access to get customer data: {:?}", auth)
        }

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let customers = customers.find_all_by_ids("user_id", ids).await?;

        Ok(customers.into_iter().map(|x| x.into()).collect::<Vec<_>>())
    }

    pub async fn find_projects(&self, ids: Vec<ObjectId>) -> anyhow::Result<Vec<PublicProject>> {
        let auth = self.context.auth();

        if !GetData::get_access(auth, ()) {
            bail!("No access to get customer data: {:?}", auth)
        }

        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let projects = projects.find_all_by_ids("id", ids).await?;

        Ok(projects
            .into_iter()
            .map(|x| x.into())
            .filter(|x: &PublicProject| x.publish_options.publish)
            .collect::<Vec<_>>())
    }
}
