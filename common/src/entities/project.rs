use std::{collections::HashMap, str::FromStr};

use mongodb::bson::{self, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::Entity;

use super::audit_request::PriceRange;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PublishOptions {
    pub publish: bool,
    pub ready_to_wait: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Project<Id> {
    pub id: Id,
    pub customer_id: Id,
    pub name: String,
    pub description: String,
    pub scope: Vec<String>,
    pub tags: Vec<String>,
    pub publish_options: PublishOptions,
    pub status: String,
    pub creator_contacts: HashMap<String, String>,
    pub last_modified: i64,
    pub price_range: PriceRange,
}

impl Project<String> {
    pub fn parse(self) -> Project<ObjectId> {
        Project {
            id: self.id.parse().unwrap(),
            customer_id: self.customer_id.parse().unwrap(),
            name: self.name,
            description: self.description,
            scope: self.scope,
            tags: self.tags,
            publish_options: self.publish_options,
            status: self.status,
            creator_contacts: self.creator_contacts,
            last_modified: self.last_modified,
            price_range: self.price_range,
        }
    }

    pub fn to_doc(self) -> Document {
        let mut document = bson::to_document(&self).unwrap();
        document.insert("kind", "project");
        document
    }
}

impl Project<ObjectId> {
    pub fn stringify(self) -> Project<String> {
        Project {
            id: self.id.to_hex(),
            customer_id: self.customer_id.to_hex(),
            name: self.name,
            description: self.description,
            scope: self.scope,
            tags: self.tags,
            publish_options: self.publish_options,
            status: self.status,
            creator_contacts: self.creator_contacts,
            last_modified: self.last_modified,
            price_range: self.price_range,
        }
    }
}

impl Entity for Project<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }

    fn timestamp(&self) -> i64 {
        self.last_modified
    }
}
