use std::collections::HashMap;

use mongodb::bson::{oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::Entity;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Customer<Id> {
    pub user_id: Id,
    pub avatar: String,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub contacts: HashMap<String, String>,
    pub tags: Vec<String>,
    pub last_modified: i64,
}

impl Customer<String> {
    pub fn parse(self) -> Customer<ObjectId> {
        Customer {
            user_id: self.user_id.parse().unwrap(),
            avatar: self.avatar,
            first_name: self.first_name,
            last_name: self.last_name,
            about: self.about,
            company: self.company,
            contacts: self.contacts,
            tags: self.tags,
            last_modified: self.last_modified,
        }
    }

    pub fn to_doc(self) -> Document {
        let mut document = mongodb::bson::to_document(&self).unwrap();
        document.insert("kind", "customer");
        document
    }
}

impl Customer<ObjectId> {
    pub fn stringify(self) -> Customer<String> {
        Customer {
            user_id: self.user_id.to_hex(),
            avatar: self.avatar,
            first_name: self.first_name,
            last_name: self.last_name,
            about: self.about,
            company: self.company,
            contacts: self.contacts,
            tags: self.tags,
            last_modified: self.last_modified,
        }
    }
}

impl Entity for Customer<ObjectId> {
    fn id(&self) -> ObjectId {
        self.user_id.clone()
    }
}
