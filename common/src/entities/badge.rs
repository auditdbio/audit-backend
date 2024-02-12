use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    repository::Entity,
    services::{API_PREFIX, AUDITORS_SERVICE, PROTOCOL},
};

use super::{audit_request::PriceRange, contacts::Contacts};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Badge<Id> {
    pub user_id: Id,
    pub avatar: String,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub free_at: String,
    pub tags: Vec<String>,
    pub contacts: Contacts,
    pub price_range: PriceRange,
    pub last_modified: i64,
    pub created_at: Option<i64>,
}

impl Badge<String> {
    pub fn parse(self) -> Badge<ObjectId> {
        Badge {
            user_id: self.user_id.parse().unwrap(),
            avatar: self.avatar,
            first_name: self.first_name,
            last_name: self.last_name,
            about: self.about,
            company: self.company,
            free_at: self.free_at,
            tags: self.tags,
            contacts: self.contacts,
            price_range: self.price_range,
            last_modified: self.last_modified,
            created_at: self.created_at,
        }
    }
}

impl Badge<ObjectId> {
    pub fn stringify(self) -> Badge<String> {
        Badge {
            user_id: self.user_id.to_hex(),
            avatar: self.avatar,
            first_name: self.first_name,
            last_name: self.last_name,
            about: self.about,
            company: self.company,
            free_at: self.free_at,
            tags: self.tags,
            contacts: self.contacts,
            price_range: self.price_range,
            last_modified: self.last_modified,
            created_at: self.created_at,
        }
    }
}

impl From<Badge<String>> for Badge<ObjectId> {
    fn from(badge: Badge<String>) -> Self {
        badge.parse()
    }
}

impl Entity for Badge<ObjectId> {
    fn id(&self) -> ObjectId {
        self.user_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicBadge {
    pub user_id: String,
    pub avatar: String,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub contacts: Contacts,
    pub free_at: String,
    pub price_range: PriceRange,
    pub kind: String,
    pub tags: Vec<String>,
}

impl From<Badge<ObjectId>> for Option<Document> {
    fn from(badge: Badge<ObjectId>) -> Self {
        let badge = badge.stringify();
        let mut document = mongodb::bson::to_document(&badge).unwrap();
        if !badge.contacts.public_contacts {
            document.remove("contacts");
        }
        document.insert("id", badge.user_id);
        document.insert(
            "request_url",
            format!(
                "{}://{}/{}/badge/data",
                PROTOCOL.as_str(),
                AUDITORS_SERVICE.as_str(),
                API_PREFIX.as_str(),
            ),
        );
        document.insert(
            "search_tags",
            badge
                .tags
                .iter()
                .map(|tag| tag.to_lowercase())
                .collect::<Vec<String>>(),
        );

        document.remove("last_modified");
        document.insert("kind", "badge");
        Some(document)
    }
}
