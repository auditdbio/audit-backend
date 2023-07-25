use mongodb::bson::{oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    repository::Entity,
    services::{CUSTOMERS_SERVICE, PROTOCOL},
};

use super::contacts::Contacts;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Customer<Id> {
    pub user_id: Id,
    pub avatar: String,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub contacts: Contacts,
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
        self.user_id
    }
}

impl From<Customer<ObjectId>> for Option<Document> {
    fn from(customer: Customer<ObjectId>) -> Self {
        let customer = customer.stringify();
        let mut document = mongodb::bson::to_document(&customer).unwrap();
        if !customer.contacts.public_contacts {
            document.remove("contacts");
        }
        document.insert("id", customer.user_id);
        document.insert(
            "request_url",
            format!(
                "{}://{}/api/customer/data",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str()
            ),
        );
        document.insert(
            "search_tags",
            customer
                .tags
                .iter()
                .map(|tag| tag.to_lowercase())
                .collect::<Vec<String>>(),
        );

        document.remove("last_modified");
        document.insert("kind", "customer");
        Some(document)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicCustomer {
    pub user_id: String,
    pub avatar: String,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub contacts: Contacts,
    pub tags: Vec<String>,
}

// impl From<Customer<ObjectId>> for PublicCustomer {
//     fn from(value: Customer<ObjectId>) -> Self {
//         PublicCustomer {
//             user_id: value.user_id.to_hex(),
//             avatar: value.avatar,
//             first_name: value.first_name,
//             last_name: value.last_name,
//             about: value.about,
//             company: value.company,
//             contacts: value.contacts,
//             tags: value.tags,
//         }
//     }
// }
