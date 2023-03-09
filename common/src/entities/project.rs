use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::{Entity, TaggableEntity};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PublishOptions {
    pub publish: bool,
    pub prise_from: String,
    pub prise_to: String,
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
}

impl Project<String> {
    pub fn parse(self) -> Project<ObjectId> {
        Project {
            id: ObjectId::from_str(&self.id).unwrap(),
            customer_id: ObjectId::from_str(&self.customer_id).unwrap(),
            name: self.name,
            description: self.description,
            scope: self.scope,
            tags: self.tags,
            publish_options: self.publish_options,
            status: self.status,
        }
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
        }
    }
}

impl Entity for Project<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

impl TaggableEntity for Project<ObjectId> {
    fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}
