use std::collections::HashMap;

use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::auditor::{Auditor, PublicAuditor},
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuditor {
    avatar: String,
    first_name: String,
    last_name: String,
    about: String,
    company: String,
    contacts: HashMap<String, String>,
    free_at: String,
    tax: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditorChange {
    avatar: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    company: Option<String>,
    contacts: Option<HashMap<String, String>>,
    free_at: Option<String>,
    tax: Option<String>,
    tags: Option<Vec<String>>,
}

pub struct AuditorService {
    context: Context,
}

impl AuditorService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, auditor: CreateAuditor) -> anyhow::Result<PublicAuditor> {
        let auth = self.context.auth();

        let Some(auditors) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No auditor repository found")
        };

        let auditor = Auditor {
            user_id: auth
                .id()
                .ok_or(anyhow::anyhow!("No user id found"))?
                .clone(),
            avatar: auditor.avatar,
            first_name: auditor.first_name,
            last_name: auditor.last_name,
            about: auditor.about,
            company: auditor.company,
            contacts: auditor.contacts,
            tags: auditor.tags,
            last_modified: Utc::now().timestamp_micros(),
            free_at: auditor.free_at,
            tax: auditor.tax,
        };

        auditors.insert(&auditor).await?;

        Ok(auditor.into())
    }

    pub async fn find(&self, id: ObjectId) -> anyhow::Result<Option<PublicAuditor>> {
        let auth = self.context.auth();

        let Some(auditors) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No auditor repository found")
        };

        let Some(auditor) = auditors.find("user_id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read::get_access(auth, &auditor) {
            bail!("User is not available to change this auditor")
        }

        Ok(Some(auditor.into()))
    }

    pub async fn change(
        &self,
        id: ObjectId,
        change: AuditorChange,
    ) -> anyhow::Result<PublicAuditor> {
        let auth = self.context.auth();

        let Some(auditors) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No auditor repository found")
        };

        let Some(mut auditor) = auditors.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No auditor found")
        };

        if !Edit::get_access(auth, &auditor) {
            bail!("User is not available to change this auditor")
        }

        if let Some(avatar) = change.avatar {
            auditor.avatar = avatar;
        }

        if let Some(first_name) = change.first_name {
            auditor.first_name = first_name;
        }

        if let Some(last_name) = change.last_name {
            auditor.last_name = last_name;
        }

        if let Some(about) = change.about {
            auditor.about = about;
        }

        if let Some(company) = change.company {
            auditor.company = company;
        }

        if let Some(contacts) = change.contacts {
            auditor.contacts = contacts;
        }

        if let Some(tags) = change.tags {
            auditor.tags = tags;
        }

        if let Some(free_at) = change.free_at {
            auditor.free_at = free_at;
        }

        if let Some(tax) = change.tax {
            auditor.tax = tax;
        }

        auditor.last_modified = Utc::now().timestamp_micros();

        auditors.delete("id", &id).await?;
        auditors.insert(&auditor).await?;

        Ok(auditor.into())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicAuditor> {
        let auth = self.context.auth();

        let Some(auditors) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No auditor repository found")
        };

        let Some(auditor) = auditors.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No auditor found")
        };

        if !Edit::get_access(auth, &auditor) {
            auditors.insert(&auditor).await?;
            bail!("User is not available to delete this auditor")
        }

        Ok(auditor.into())
    }
}
