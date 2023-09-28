use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{self, codes::post_code, requests::PublicRequest},
    context::Context,
    entities::{
        audit_request::PriceRange,
        auditor::PublicAuditor,
        badge::{Badge, PublicBadge},
        contacts::Contacts,
        letter::CreateLetter,
    },
    error::{self, AddCode},
    services::AUDITORS_SERVICE,
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBadge {
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
pub struct BadgeChange {
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

#[derive(Debug, Serialize, Deserialize)]
struct CodePayload {
    badge_id: ObjectId,
    user_id: ObjectId,
}

pub struct BadgeService {
    context: Context,
}

impl BadgeService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, badge: CreateBadge) -> error::Result<Badge<String>> {
        let _auth = self.context.auth();

        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;

        let badge = Badge {
            user_id: ObjectId::new(),
            avatar: badge.avatar.unwrap_or_default(),
            first_name: badge.first_name,
            last_name: badge.last_name,
            about: badge.about.unwrap_or_default(),
            company: badge.company.unwrap_or_default(),
            contacts: badge.contacts,
            tags: badge.tags.unwrap_or_default(),
            last_modified: Utc::now().timestamp_micros(),
            free_at: badge.free_at.unwrap_or_default(),
            price_range: badge.price_range.unwrap_or_default(),
        };

        badges.insert(&badge).await?;

        Ok(badge.stringify())
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicBadge>> {
        let auth = self.context.auth();

        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;

        let Some(badge) = badges.find("user_id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(auth, &badge) {
            return Err(anyhow::anyhow!("User is not available to change this badge").code(400));
        }

        Ok(Some(auth.public_badge(badge)))
    }

    pub async fn change(&self, change: BadgeChange) -> error::Result<Badge<String>> {
        let auth = self.context.auth();
        let id = *auth.id().unwrap();

        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;

        let Some(mut badge) = badges.find("user_id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No badge found").code(400));
        };

        if !Edit.get_access(auth, &badge) {
            return Err(anyhow::anyhow!("User is not available to change this badge").code(400));
        }

        if let Some(avatar) = change.avatar {
            badge.avatar = avatar;
        }

        if let Some(first_name) = change.first_name {
            badge.first_name = first_name;
        }

        if let Some(last_name) = change.last_name {
            badge.last_name = last_name;
        }

        if let Some(about) = change.about {
            badge.about = about;
        }

        if let Some(company) = change.company {
            badge.company = company;
        }

        if let Some(contacts) = change.contacts {
            badge.contacts = contacts;
        }

        if let Some(tags) = change.tags {
            badge.tags = tags;
        }

        if let Some(free_at) = change.free_at {
            badge.free_at = free_at;
        }

        if let Some(price_range) = change.price_range {
            badge.price_range = price_range;
        }

        badge.last_modified = Utc::now().timestamp_micros();

        badges.delete("user_id", &id).await?;
        badges.insert(&badge).await?;

        Ok(badge.stringify())
    }

    pub async fn substitute(&self, badge_id: ObjectId, user_id: ObjectId) -> error::Result<()> {
        let auth = self.context.auth();

        let payload = CodePayload { badge_id, user_id };
        // create code
        let code = post_code(
            &self.context,
            auth.clone(),
            serde_json::to_string(&payload)?,
        )
        .await?;

        // get user email
        let user = api::user::get_by_id(&self.context, user_id).await?; // Wrong email here

        // send link with code
        let link = format!(
            "https://{}/api/badge/substitute/run/{}",
            AUDITORS_SERVICE.as_str(),
            code
        );

        let letter = CreateLetter {
            email: user.email,
            message: include_str!("../../templates/code.txt").replace("{link}", &link),
            subject: include_str!("../../templates/code_subject.txt").to_owned(),

            ..CreateLetter::default()
        };

        api::mail::send_mail(&self.context, letter).await?;

        Ok(())
    }

    pub async fn substitute_run(&self, code: String) -> error::Result<PublicAuditor> {
        let auth = self.context.auth();
        // get payload from code
        let payload: CodePayload = serde_json::from_str(
            &api::codes::get_code(&self.context, auth.clone(), code)
                .await?
                .unwrap(),
        )?;
        // get all audit requests

        let mut requests: Vec<PublicRequest> = vec![];

        // delete all audit requests

        for request in &requests {
            // api::requests::delete_request(&self.context, request.id).await?;
        }
        // create new audit requests

        for request in &mut requests {
            // CreateRequest
            // api::requests::create_request(&self.context, request).await?;
        }

        // create auditor

        // delete badge

        todo!()
    }

    pub async fn delete(&self, badge_id: ObjectId) -> error::Result<PublicBadge> {
        // send code
        todo!()
    }

    pub async fn delete_run(&self, code: String) -> error::Result<PublicBadge> {
        // send code
        todo!()
    }
}
