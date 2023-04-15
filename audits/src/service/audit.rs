use std::collections::HashMap;

use anyhow::bail;
use chrono::Utc;
use common::{context::Context, entities::{customer::Customer, audit::Audit}, access_rules::{Edit, AccessRules, Read}};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAudit {
    customer_id: String,
    auditor_id: String,
    project_id: String,
    name: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditChange {
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicAudit {

}


impl From<Customer<ObjectId>> for PublicAudit {
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

    pub async fn create(&self, audit: CreateAudit) -> anyhow::Result<PublicAudit> {
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

    pub async fn find(&self, id: ObjectId) -> anyhow::Result<Option<PublicAudit>> {
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

    pub async fn change(&self, id: ObjectId, change: AuditChange) -> anyhow::Result<PublicAudit> {
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

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicAudit> {
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
