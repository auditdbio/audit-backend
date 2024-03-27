use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    repository::Entity,
    services::{API_PREFIX, AUDITORS_SERVICE, PROTOCOL},
};

use super::{audit_request::PriceRange, badge::PublicBadge, contacts::Contacts};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Auditor<Id> {
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
    pub link_id: Option<String>,
}

impl Auditor<String> {
    pub fn parse(self) -> Auditor<ObjectId> {
        Auditor {
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
            link_id: self.link_id,
        }
    }
}

impl Auditor<ObjectId> {
    pub fn stringify(self) -> Auditor<String> {
        Auditor {
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
            link_id: self.link_id,
        }
    }
}

impl From<Auditor<String>> for Auditor<ObjectId> {
    fn from(auditor: Auditor<String>) -> Self {
        auditor.parse()
    }
}

impl Entity for Auditor<ObjectId> {
    fn id(&self) -> ObjectId {
        self.user_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicAuditor {
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
    pub link_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "api_kind")]
pub enum ExtendedAuditor {
    Auditor(PublicAuditor),
    Badge(PublicBadge),
}

impl ExtendedAuditor {
    pub fn avatar(&self) -> &String {
        match self {
            ExtendedAuditor::Auditor(auditor) => &auditor.avatar,
            ExtendedAuditor::Badge(badge) => &badge.avatar,
        }
    }

    pub fn first_name(&self) -> &String {
        match self {
            ExtendedAuditor::Auditor(auditor) => &auditor.first_name,
            ExtendedAuditor::Badge(badge) => &badge.first_name,
        }
    }

    pub fn last_name(&self) -> &String {
        match self {
            ExtendedAuditor::Auditor(auditor) => &auditor.last_name,
            ExtendedAuditor::Badge(badge) => &badge.last_name,
        }
    }

    pub fn contacts(&self) -> &Contacts {
        match self {
            ExtendedAuditor::Auditor(auditor) => &auditor.contacts,
            ExtendedAuditor::Badge(badge) => &badge.contacts,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ExtendedAuditor::Auditor(auditor) => auditor.first_name.is_empty(),
            ExtendedAuditor::Badge(badge) => badge.first_name.is_empty(),
        }
    }
}

impl From<Auditor<ObjectId>> for Option<Document> {
    fn from(auditor: Auditor<ObjectId>) -> Self {
        let auditor = auditor.stringify();
        let mut document = mongodb::bson::to_document(&auditor).unwrap();
        if !auditor.contacts.public_contacts {
            document.remove("contacts");
        }
        document.insert("id", auditor.user_id);
        document.insert(
            "request_url",
            format!(
                "{}://{}/{}/auditor/data",
                PROTOCOL.as_str(),
                AUDITORS_SERVICE.as_str(),
                API_PREFIX.as_str(),
            ),
        );
        document.insert(
            "search_tags",
            auditor
                .tags
                .iter()
                .map(|tag| tag.to_lowercase())
                .collect::<Vec<String>>(),
        );

        document.remove("last_modified");
        document.insert("kind", "auditor");
        Some(document)
    }
}

// impl From<Auditor<ObjectId>> for PublicAuditor {
//     fn from(auditor: Auditor<ObjectId>) -> Self {
//         let auditor = auditor.stringify();
//         PublicAuditor {
//             user_id: auditor.user_id,
//             avatar: auditor.avatar,
//             first_name: auditor.first_name,
//             last_name: auditor.last_name,
//             about: auditor.about,
//             company: auditor.company,
//             contacts: auditor.contacts,
//             free_at: auditor.free_at,
//             price_range: auditor.price_range,
//             tags: auditor.tags,
//         }
//     }
// }
