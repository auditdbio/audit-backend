use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    repository::Entity,
    services::{AUDITORS_SERVICE, PROTOCOL},
};

use super::{audit_request::PriceRange, contacts::Contacts};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Bage<Id> {
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
}

impl Bage<String> {
    pub fn parse(self) -> Bage<ObjectId> {
        Bage {
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
        }
    }
}

impl Bage<ObjectId> {
    pub fn stringify(self) -> Bage<String> {
        Bage {
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
        }
    }
}

impl From<Bage<String>> for Bage<ObjectId> {
    fn from(bage: Bage<String>) -> Self {
        bage.parse()
    }
}

impl Entity for Bage<ObjectId> {
    fn id(&self) -> ObjectId {
        self.user_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicBage {
    pub user_id: String,
    pub avatar: String,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub contacts: Contacts,
    pub free_at: String,
    pub price_range: PriceRange,
    pub tags: Vec<String>,
}

impl From<Bage<ObjectId>> for Option<Document> {
    fn from(bage: Bage<ObjectId>) -> Self {
        let bage = bage.stringify();
        let mut document = mongodb::bson::to_document(&bage).unwrap();
        if !bage.contacts.public_contacts {
            document.remove("contacts");
        }
        document.insert("id", bage.user_id);
        document.insert(
            "request_url",
            format!(
                "{}://{}/api/bage/data",
                PROTOCOL.as_str(),
                AUDITORS_SERVICE.as_str()
            ),
        );
        document.insert(
            "search_tags",
            bage.tags
                .iter()
                .map(|tag| tag.to_lowercase())
                .collect::<Vec<String>>(),
        );

        document.remove("last_modified");
        document.insert("kind", "bage");
        Some(document)
    }
}
