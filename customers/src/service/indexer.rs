use common::{
    access_rules::{AccessRules, GetData},
    context::Context,
    entities::{
        customer::{Customer, PublicCustomer},
        project::{Project, PublicProject},
    },
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

    pub async fn index_customer(&self, since: i64) -> error::Result<Vec<Document>> {
        let auth = self.context.auth();

        if !GetData.get_access(auth, ()) {
            return Err(anyhow::anyhow!("No access to get customer data {:?}", auth).code(400));
        }

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        let customers = customers.get_all_since(since).await?;

        Ok(customers
            .into_iter()
            .filter_map(|x| x.into())
            .collect::<Vec<_>>())
    }

    pub async fn index_project(&self, since: i64) -> error::Result<Vec<Document>> {
        let auth = self.context.auth();

        if !GetData.get_access(auth, ()) {
            return Err(anyhow::anyhow!("No access to get customer data: {:?}", auth).code(400));
        }
        let customers = self.context.try_get_repository::<Project<ObjectId>>()?;

        let customers = customers.get_all_since(since).await?;

        Ok(customers
            .into_iter()
            .filter_map(|x| x.into())
            .collect::<Vec<_>>())
    }

    pub async fn find_customers(&self, ids: Vec<ObjectId>) -> error::Result<Vec<PublicCustomer>> {
        let auth = self.context.auth();

        if !GetData.get_access(auth, ()) {
            return Err(anyhow::anyhow!("No access to get customer data: {:?}", auth).code(400));
        }

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        let customers = customers.find_all_by_ids("user_id", ids).await?;

        Ok(customers
            .into_iter()
            .map(|x| auth.public_customer(x))
            .collect::<Vec<_>>())
    }

    pub async fn find_projects(&self, ids: Vec<ObjectId>) -> error::Result<Vec<PublicProject>> {
        let auth = self.context.auth();

        if !GetData.get_access(auth, ()) {
            return Err(anyhow::anyhow!("No access to get customer data: {:?}", auth).code(400));
        }

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let projects = projects.find_all_by_ids("id", ids).await?;

        Ok(projects
            .into_iter()
            .map(|x| auth.public_project(x))
            .filter(|x: &PublicProject| x.publish_options.publish)
            .collect::<Vec<_>>())
    }
}
