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
    pub public_contacts: bool,
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
            public_contacts: self.public_contacts,
            contacts: self.contacts,
            tags: self.tags,
            last_modified: self.last_modified,
        }
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
            public_contacts: self.public_contacts,
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

impl From<Customer<ObjectId>> for Option<Document> {
    fn from(customer: Customer<ObjectId>) -> Self {
        let customer = customer.stringify();
        let mut document = mongodb::bson::to_document(&customer).unwrap();
        if !customer.public_contacts {
            document.remove("contacts");
        }
        document.remove("last_modified");
        document.insert("kind", "customer");
        Some(document)
    }
}
