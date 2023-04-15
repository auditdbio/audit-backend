use std::collections::HashMap;

use anyhow::bail;
use chrono::Utc;
use common::{context::Context, entities::{customer::Customer, audit::Audit, audit_request::{TimeRange, PriceRange}}, access_rules::{Edit, AccessRules, Read}};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRequest {
    customer_id: String,
    auditor_id: String,
    project_id: String,

    description: String,
    time: TimeRange,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestChange {
    auditor_id: Option<String>,
    project_id: Option<String>,
    description: Option<String>,
    time: Option<TimeRange>,
    project_name: Option<String>,
    project_avatar: Option<String>,
    project_description: Option<String>,
    project_scope: Option<Vec<String>>,
    price_range: Option<PriceRange>,
    price: Option<i64>,
    auditor_contacts: Option<HashMap<String, String>>,
    customer_contacts: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicRequest {

}


impl From<Customer<ObjectId>> for PublicRequest {
    fn from(customer: Customer<ObjectId>) -> Self {
        Self {
        }
    }
}


pub struct AuditService {
    context: Context,
}

impl AuditService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, audit: CreateRequest) -> anyhow::Result<PublicRequest> {
        let auth = self.context.auth_res()?;

        let Some(audits) = self.context.get_repository::<Audit<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let audit = Audit {
            
            last_modified: Utc::now().timestamp_micros(),
        };

        audits.insert(&audit).await?;

        Ok(audit.into())
    }

    pub async fn find(&self, id: ObjectId) -> anyhow::Result<Option<PublicRequest>> {
        let auth = self.context.auth_res()?;

        let Some(audits) = self.context.get_repository::<Audit<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(audit) = audits.find("user_id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read::get_access(&auth, &audit) {
            bail!("User is not available to change this customer")
        }

        Ok(Some(audit.into()))
    }

    pub async fn change(&self, id: ObjectId, change: RequestChange) -> anyhow::Result<PublicRequest> {
        let auth = self.context.auth_res()?;

        let Some(audits) = self.context.get_repository::<Audit<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(mut audit) = audits.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No customer found")
        };

        if !Edit::get_access(&auth, &audit) {
            bail!("User is not available to change this customer")
        }

        // TODO: Change audit here

        audit.last_modified = Utc::now().timestamp_micros();

        audits.delete("id", &id).await?;
        audits.insert(&audit).await?;

        Ok(audit.into())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicRequest> {
        let auth = self.context.auth_res()?;

        let Some(audits) = self.context.get_repository::<Audit<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(audit) = audits.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No customer found")
        };

        if !Edit::get_access(&auth, &audit) {
            audits.insert(&audit).await?;
            bail!("User is not available to delete this customer")
        }

        Ok(audit.into())
    }
}
