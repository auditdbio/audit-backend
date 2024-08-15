use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{
        self,
        badge::BadgePayload,
        codes::post_code,
        mail::send_mail,
        requests::{get_audit_requests, CreateRequest},
        seartch::delete_from_search,
        user::get_by_email,
    },
    auth::Auth,
    context::GeneralContext,
    entities::{
        audit_request::PriceRange,
        auditor::{Auditor, PublicAuditor},
        badge::{Badge, PublicBadge},
        contacts::Contacts,
        letter::CreateLetter,
    },
    error::{self, AddCode},
    services::{FRONTEND, PROTOCOL},
};
use mongodb::bson::{oid::ObjectId, Bson, doc};
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
    link_id: Option<String>,
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

pub struct BadgeService {
    context: GeneralContext,
}

impl BadgeService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn create(&self, badge: CreateBadge) -> error::Result<Badge<String>> {
        let _auth = self.context.auth();

        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;

        let old_badge = badges
            .find(
                "contacts.email",
                &Bson::String(badge.contacts.email.clone().unwrap()),
            )
            .await?;

        if let Some(_badge) = old_badge {
            return Err(anyhow::anyhow!("Badge already exists").code(400));
        };

        let user = get_by_email(&self.context, badge.contacts.email.clone().unwrap()).await?;

        if let Some(_user) = user {
            return Err(anyhow::anyhow!("User already exists").code(400));
        };

        let id = ObjectId::new();
        let badge = Badge {
            user_id: id.clone(),
            avatar: badge.avatar.unwrap_or_default(),
            first_name: badge.first_name,
            last_name: badge.last_name,
            about: badge.about.unwrap_or_default(),
            company: badge.company.unwrap_or_default(),
            contacts: badge.contacts,
            tags: badge.tags.unwrap_or_default(),
            last_modified: Utc::now().timestamp_micros(),
            created_at: Some(Utc::now().timestamp_micros()),
            free_at: badge.free_at.unwrap_or_default(),
            price_range: badge.price_range.unwrap_or_default(),
            link_id: badge.link_id.map(|id| id.to_lowercase()).or_else(|| Some(id.to_hex())),
        };

        let payload = BadgePayload {
            badge_id: badge.user_id,
            email: badge.contacts.email.clone().unwrap(),
        };

        let code = post_code(&self.context, serde_json::to_string(&payload)?).await?;

        // delete link
        let delete_link = format!(
            "{}://{}/delete/{}/{}",
            PROTOCOL.as_str(),
            FRONTEND.as_str(),
            badge.user_id,
            code
        );
        // merge link
        let merge_link = format!(
            "{}://{}/invite-user/{}/{}",
            PROTOCOL.as_str(),
            FRONTEND.as_str(),
            badge.user_id,
            code
        );

        let message = include_str!("../../templates/new_user.txt")
            .replace("{delete_link}", &delete_link)
            .replace("{merge_link}", &merge_link);

        // send email
        let letter = CreateLetter {
            recipient_id: None,
            recipient_name: None,
            email: badge.contacts.email.clone().unwrap(),
            message: message.clone(),
            subject: "AuditDB Introduction".to_string(),
        };
        send_mail(&self.context, letter).await?;

        badges.insert(&badge).await?;

        Ok(badge.stringify())
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicBadge>> {
        let auth = self.context.auth();

        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;

        let Some(badge) = badges.find("user_id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(&auth, &badge) {
            return Err(anyhow::anyhow!("User is not available to change this badge").code(400));
        }

        Ok(Some(auth.public_badge(badge)))
    }

    pub async fn find_by_email(&self, email: String) -> error::Result<Option<PublicBadge>> {
        let auth = self.context.auth();

        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;

        let Some(badge) = badges.find("contacts.email", &Bson::String(email)).await? else {
            return Ok(None);
        };

        if !Read.get_access(&auth, &badge) {
            return Err(anyhow::anyhow!("User is not available to change this badge").code(400));
        }

        Ok(Some(auth.public_badge(badge)))
    }

    pub async fn change(&self, change: BadgeChange) -> error::Result<Badge<String>> {
        let auth = self.context.auth();
        let id = auth.id().unwrap();

        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;

        let Some(mut badge) = badges.find("user_id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No badge found").code(400));
        };

        if !Edit.get_access(&auth, &badge) {
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
        // badges.update_one(doc! {"user_id": &id}, &badge).await?;

        Ok(badge.stringify())
    }

    pub async fn merge(&self, code: String) -> error::Result<PublicAuditor> {
        let auth = self.context.auth();

        let id = auth.id().unwrap();
        let id_str = id.to_hex();

        // get payload from code
        let payload: BadgePayload =
            serde_json::from_str(&api::codes::get_code(&self.context, code).await?.unwrap())?;

        let Some(badge) = self.find(payload.badge_id).await? else {
            return Err(anyhow::anyhow!("No badge found").code(400));
        };
        // get all audit requests

        let mut requests =
            get_audit_requests(&self.context, Auth::User(badge.user_id.parse()?)).await?;

        // delete all audit requests

        for request in &requests {
            api::requests::delete(
                &self.context,
                self.context.server_auth(),
                request.id.parse()?,
            )
            .await?;
        }

        let auditors = self.context.try_get_repository::<Auditor<ObjectId>>()?;
        // create auditor
        let auditor = Auditor {
            user_id: id,
            avatar: badge.avatar,
            first_name: badge.first_name,
            last_name: badge.last_name,
            about: badge.about,
            company: badge.company,
            contacts: badge.contacts,
            tags: badge.tags,
            last_modified: Utc::now().timestamp_micros(),
            created_at: Some(Utc::now().timestamp_micros()),
            free_at: badge.free_at,
            price_range: badge.price_range,
            link_id: badge.link_id,
            rating: None,
        };

        if auditors
            .find("user_id", &Bson::ObjectId(id))
            .await?
            .is_none()
        {
            auditors.insert(&auditor).await?;
        }

        // delete badge
        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;
        badges.delete("user_id", &payload.badge_id).await?;

        delete_from_search(&self.context, payload.badge_id).await?;

        // create new audit requests

        for request in &mut requests {
            if &request.customer_id == &id_str {
                continue;
            }
            // CreateRequest
            let new_request = CreateRequest {
                customer_id: request.customer_id.clone(),
                auditor_id: id_str.clone(),
                project_id: request.project_id.clone(),
                price: request.price.clone(),
                total_cost: request.total_cost.clone(),
                description: request.description.clone(),
                time: request.time.clone(),
                auditor_organization: request.auditor_organization.clone().map(|org| org.id),
                customer_organization: request.customer_organization.clone().map(|org| org.id),
            };

            let auth = Auth::User(request.customer_id.parse()?);

            api::requests::create_request(&self.context, auth, new_request).await?;
        }

        Ok(auth.public_auditor(auditor))
    }

    pub async fn delete(&self, code: String) -> error::Result<()> {
        let payload: BadgePayload =
            serde_json::from_str(&api::codes::get_code(&self.context, code).await?.unwrap())?;
        let auth = Auth::User(payload.badge_id);

        // get all audit requests

        let requests = get_audit_requests(&self.context, auth).await?;

        // delete all audit requests

        for request in &requests {
            api::requests::delete(
                &self.context,
                self.context.server_auth(),
                request.id.parse()?,
            )
            .await?;
        }

        // delete badge
        let badges = self.context.try_get_repository::<Badge<ObjectId>>()?;
        badges.delete("user_id", &payload.badge_id).await?;

        delete_from_search(&self.context, payload.badge_id).await?;

        Ok(())
    }
}
