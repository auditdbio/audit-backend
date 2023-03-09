use std::{collections::HashMap, str::FromStr};

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::{Entity, TaggableEntity};

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
    pub contacts: HashMap<String, String>,
    pub tax: String,
}

impl Auditor<String> {
    pub fn parse(self) -> Auditor<ObjectId> {
        Auditor {
            user_id: ObjectId::from_str(&self.user_id).unwrap(),
            avatar: self.avatar,
            first_name: self.first_name,
            last_name: self.last_name,
            about: self.about,
            company: self.company,
            free_at: self.free_at,
            tags: self.tags,
            contacts: self.contacts,
            tax: self.tax,
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
            tax: self.tax,
        }
    }
}

impl Entity for Auditor<ObjectId> {
    fn id(&self) -> ObjectId {
        self.user_id.clone()
    }
}

impl TaggableEntity for Auditor<ObjectId> {
    fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}
