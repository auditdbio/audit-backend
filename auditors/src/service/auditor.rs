use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{
        audit_request::PriceRange,
        auditor::{Auditor, PublicAuditor},
        contacts::Contacts,
        user::PublicUser,
    },
    services::{PROTOCOL, USERS_SERVICE},
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuditor {
    avatar: Option<String>,
    first_name: String,
    last_name: String,
    about: Option<String>,
    company: Option<String>,
    contacts: Contacts,
    free_at: Option<String>,
    price_range: Option<PriceRange>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditorChange {
    avatar: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    company: Option<String>,
    contacts: Option<Contacts>,
    free_at: Option<String>,
    price_range: Option<PriceRange>,
    tags: Option<Vec<String>>,
}

pub struct AuditorService {
    context: Context,
}

impl AuditorService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, auditor: CreateAuditor) -> anyhow::Result<Auditor<String>> {
        let auth = self.context.auth();

        let Some(auditors) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No auditor repository found")
        };

        let auditor = Auditor {
            user_id: auth
                .id()
                .ok_or(anyhow::anyhow!("No user id found"))?
                .clone(),
            avatar: auditor.avatar.unwrap_or_default(),
            first_name: auditor.first_name,
            last_name: auditor.last_name,
            about: auditor.about.unwrap_or_default(),
            company: auditor.company.unwrap_or_default(),
            contacts: auditor.contacts,
            tags: auditor.tags.unwrap_or_default(),
            last_modified: Utc::now().timestamp_micros(),
            free_at: auditor.free_at.unwrap_or_default(),
            price_range: auditor.price_range.unwrap_or_default(),
            is_new: true,
        };

        auditors.insert(&auditor).await?;

        Ok(auditor.stringify())
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

        Ok(Some(auth.public_auditor(auditor)))
    }

    pub async fn my_auditor(&self) -> anyhow::Result<Option<Auditor<String>>> {
        let auth = self.context.auth();

        let Some(auditors) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No auditor repository found")
        };

        let auditor = auditors
            .find("user_id", &Bson::ObjectId(auth.id().unwrap().clone()))
            .await?
            .map(Auditor::stringify);

        if let None = auditor {
            let user = self
                .context
                .make_request::<PublicUser>()
                .auth(auth.clone())
                .get(format!(
                    "{}://{}/api/user/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    auth.id().unwrap()
                ))
                .send()
                .await?
                .json::<PublicUser>()
                .await?;

            let mut iter = user.name.split(' ');

            let first_name = iter.next().unwrap();
            let last_name = iter.next().unwrap_or_default();

            let auditor = CreateAuditor {
                avatar: None,
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
                about: None,
                company: None,
                contacts: Contacts {
                    email: Some(user.email),
                    telegram: None,
                    public_contacts: true,
                },
                tags: None,
                free_at: None,
                price_range: None,
            };

            let auditor = self.create(auditor).await?;

            return Ok(Some(auditor));
        }

        Ok(auditor)
    }

    pub async fn change(&self, change: AuditorChange) -> anyhow::Result<Auditor<String>> {
        let auth = self.context.auth();
        let id = auth.id().unwrap().clone();

        let Some(auditors) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No auditor repository found")
        };

        let Some(mut auditor) = auditors.find("user_id", &Bson::ObjectId(id)).await? else {
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

        if let Some(price_range) = change.price_range {
            auditor.price_range = price_range;
        }

        auditor.is_new = false;

        auditor.last_modified = Utc::now().timestamp_micros();

        auditors.delete("user_id", &id).await?;
        auditors.insert(&auditor).await?;

        Ok(auditor.stringify())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicAuditor> {
        let auth = self.context.auth();

        let Some(auditors) = self.context.get_repository::<Auditor<ObjectId>>() else {
            bail!("No auditor repository found")
        };

        let Some(auditor) = auditors.delete("user_id", &id).await? else {
            bail!("No auditor found")
        };

        if !Edit::get_access(auth, &auditor) {
            auditors.insert(&auditor).await?;
            bail!("User is not available to delete this auditor")
        }

        Ok(auth.public_auditor(auditor))
    }
}
