use mongodb::bson::{oid::ObjectId, Document};
use common::{
    access_rules::{AccessRules, GetData},
    context::GeneralContext,
    entities::organization::{Organization, PublicOrganization},
    error::{self, AddCode},
};

pub struct IndexerService {
    context: GeneralContext,
}

impl IndexerService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn index_organization(&self, since: i64) -> error::Result<Vec<Document>> {
        let auth = self.context.auth();

        if !GetData.get_access(&auth, ()) {
            return Err(anyhow::anyhow!("No access to get organization data {:?}", auth).code(400));
        }

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let organizations = organizations.get_all_since(since).await?;

        Ok(organizations
            .into_iter()
            .filter_map(|x| x.into())
            .collect::<Vec<_>>())
    }

    pub async fn find_organizations(&self, ids: Vec<ObjectId>) -> error::Result<Vec<PublicOrganization>> {
        let auth = self.context.auth();

        if !GetData.get_access(&auth, ()) {
            return Err(anyhow::anyhow!("No access to get organization data: {:?}", auth).code(400));
        }

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let organizations = organizations.find_all_by_ids("id", ids).await?;

        let mut result = vec![];

        for org in organizations {
            let public_org = PublicOrganization::new(&self.context, org.clone(), false).await?;
            result.push(public_org);
        }

        Ok(result)
    }
}