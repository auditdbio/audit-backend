use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{seartch::delete_from_search, user::{get_by_id, new_link_id, validate_name}},
    context::GeneralContext,
    entities::{
        audit_request::PriceRange,
        auditor::{Auditor, ExtendedAuditor, PublicAuditor},
        badge::Badge,
        contacts::Contacts,
        customer::PublicCustomer,
    },
    error::{self, AddCode},
    services::{API_PREFIX, PROTOCOL, USERS_SERVICE},
};
use mongodb::bson::{oid::ObjectId, Bson, doc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuditor {
    pub avatar: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub about: Option<String>,
    pub company: Option<String>,
    pub contacts: Contacts,
    pub free_at: Option<String>,
    pub price_range: Option<PriceRange>,
    pub tags: Option<Vec<String>>,
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
    link_id: Option<String>,
    rating: Option<f32>,
}

pub struct AuditorService {
    context: GeneralContext,
}

impl AuditorService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn create(&self, auditor: CreateAuditor) -> error::Result<Auditor<String>> {
        let auth = self.context.auth();
        let id = auth.id().ok_or(anyhow::anyhow!("No user id found"))?;

        let user = get_by_id(&self.context, auth, id.clone()).await?;

        let auditors = self.context.try_get_repository::<Auditor<ObjectId>>()?;

        let link_id = new_link_id(
            &self.context,
            user.name,
            id.clone(),
            true,
        ).await?;

        let auditor = Auditor {
            user_id: id,
            avatar: auditor.avatar.unwrap_or_default(),
            first_name: auditor.first_name,
            last_name: auditor.last_name,
            about: auditor.about.unwrap_or_default(),
            company: auditor.company.unwrap_or_default(),
            contacts: auditor.contacts,
            tags: auditor.tags.unwrap_or_default(),
            last_modified: Utc::now().timestamp_micros(),
            created_at: Some(Utc::now().timestamp_micros()),
            free_at: auditor.free_at.unwrap_or_default(),
            price_range: auditor.price_range.unwrap_or_default(),
            link_id: Some(link_id),
            rating: None,
        };

        auditors.insert(&auditor).await?;

        Ok(auditor.stringify())
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<ExtendedAuditor>> {
        let auth = self.context.auth();

        let auditors = self.context.try_get_repository::<Auditor<ObjectId>>()?;

        let Some(auditor) = auditors.find("user_id", &Bson::ObjectId(id)).await? else {
            let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;

            let Some(badge) = badges.find("user_id", &Bson::ObjectId(id)).await? else {
                return Ok(None);
            };

            return Ok(Some(ExtendedAuditor::Badge(auth.public_badge(badge))));
        };

        if !Read.get_access(&auth, &auditor) {
            return Err(anyhow::anyhow!("User is not available to change this auditor").code(400));
        }

        Ok(Some(ExtendedAuditor::Auditor(auth.public_auditor(auditor))))
    }

    pub async fn find_by_link_id(&self, link_id: String) -> error::Result<ExtendedAuditor> {
        let auth = self.context.auth();

        let auditors = self.context.try_get_repository::<Auditor<ObjectId>>()?;

        if let Some(auditor) = auditors
            .find("link_id", &Bson::String(link_id.clone().to_lowercase()))
            .await? {
            return Ok(ExtendedAuditor::Auditor(auth.public_auditor(auditor)));
        }

        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;
        if let Some(badge) = badges
            .find("link_id", &Bson::String(link_id.clone().to_lowercase()))
            .await? {
            return Ok(ExtendedAuditor::Badge(auth.public_badge(badge)));
        }

        let id = link_id
            .to_lowercase()
            .parse()
            .map_err(|_| anyhow::anyhow!("Auditor not found").code(404))?;

        if let Some(auditor) = self.find(id).await? {
            return Ok(auditor);
        }

        Err(anyhow::anyhow!("Auditor not found").code(404))
    }

    pub async fn my_auditor(&self) -> error::Result<Option<Auditor<String>>> {
        let auth = self.context.auth();

        let auditors = self.context.try_get_repository::<Auditor<ObjectId>>()?;

        let auditor = auditors
            .find("user_id", &Bson::ObjectId(auth.id().unwrap()))
            .await?
            .map(Auditor::stringify);

        if auditor.is_none() {
            let user = get_by_id(&self.context, auth, auth.id().unwrap()).await?;

            if user.current_role.to_lowercase() != "auditor" {
                return Ok(None);
            }

            let has_customer = self
                .context
                .make_request::<PublicCustomer>()
                .auth(auth)
                .get(format!(
                    "{}://{}/{}/customer/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    auth.id().unwrap()
                ))
                .send()
                .await?
                .json::<PublicCustomer>()
                .await
                .is_ok();

            if has_customer {
                return Ok(None);
            }

            let mut iter = user.name.split(|c| c == '_' || c == '-');

            let first_name = iter.next().unwrap();
            let last_name = iter.last().unwrap_or_default();

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

    pub async fn change(&self, change: AuditorChange) -> error::Result<Auditor<String>> {
        let auth = self.context.auth();
        let id = auth.id().unwrap();

        let auditors = self.context.try_get_repository::<Auditor<ObjectId>>()?;

        let Some(mut auditor) = auditors
            .find("user_id", &Bson::ObjectId(id.clone()))
            .await? else {
            return Err(anyhow::anyhow!("No auditor found").code(400));
        };

        if !Edit.get_access(&auth, &auditor) {
            return Err(anyhow::anyhow!("User is not available to change this auditor").code(400));
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

        if let Some(link_id) = change.link_id {
            if !validate_name(&link_id) {
                return Err(
                    anyhow::anyhow!("Link ID may only contain alphanumeric characters, hyphens or underscore")
                        .code(400)
                );
            }

            let new_link_id = new_link_id(
                &self.context,
                link_id,
                id,
                false,
            ).await?;

            auditor.link_id = Some(new_link_id)
        }

        auditor.last_modified = Utc::now().timestamp_micros();

        auditors.delete("user_id", &id).await?;
        auditors.insert(&auditor).await?;
        // auditors.update_one(doc! {"user_id": &id}, &auditor).await?;

        Ok(auditor.stringify())
    }

    pub async fn change_by_id(
        &self,
        id: ObjectId,
        change: AuditorChange
    ) -> error::Result<Auditor<String>> {
        let auth = self.context.auth();

        if !auth.full_access() {
            return Err(anyhow::anyhow!("User is not available to change this auditor").code(400));
        }

        let auditors = self.context.try_get_repository::<Auditor<ObjectId>>()?;
        let Some(mut auditor) = auditors.find("user_id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No auditor found").code(400));
        };

        if change.link_id.is_some() {
            auditor.link_id = change.link_id;
        }

        if change.rating.is_some() {
            auditor.rating = change.rating;
        }

        auditor.last_modified = Utc::now().timestamp_micros();

        auditors.delete("user_id", &id).await?;
        auditors.insert(&auditor).await?;
        // auditors.update_one(doc! {"user_id": &id}, &auditor).await?;

        Ok(auditor.stringify())
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<PublicAuditor> {
        let auth = self.context.auth();

        let auditors = self.context.try_get_repository::<Auditor<ObjectId>>()?;

        let Some(auditor) = auditors.delete("user_id", &id).await? else {
            return Err(anyhow::anyhow!("No auditor found").code(400));
        };

        if !Edit.get_access(&auth, &auditor) {
            auditors.insert(&auditor).await?;
            return Err(anyhow::anyhow!("User is not available to delete this auditor").code(400));
        }
        delete_from_search(&self.context, id).await?;

        Ok(auth.public_auditor(auditor))
    }
}
