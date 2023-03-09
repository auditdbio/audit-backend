use std::{collections::HashMap, str::FromStr};

use mongodb::bson::oid::ObjectId;
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
}

impl Customer<String> {
    pub fn parse(self) -> Customer<ObjectId> {
        Customer {
            user_id: ObjectId::from_str(&self.user_id).unwrap(),
            avatar: self.avatar,
            first_name: self.first_name,
            last_name: self.last_name,
            about: self.about,
            company: self.company,
            contacts: self.contacts,
            tags: self.tags,
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
        }
    }
}

impl Entity for Customer<ObjectId> {
    fn id(&self) -> ObjectId {
        self.user_id.clone()
    }
}
