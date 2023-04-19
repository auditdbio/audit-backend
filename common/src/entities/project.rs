use std::collections::HashMap;

use mongodb::bson::{self, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::Entity;

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
    pub price: i64,
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
            price: self.price,
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
            creator_contacts: self.creator_contacts,
            last_modified: self.last_modified,
            price: self.price,
        }
    }
}

impl Entity for Project<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicProject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: Vec<String>,
    pub tags: Vec<String>,
    pub publish_options: PublishOptions,
    pub status: String,
    pub creator_contacts: HashMap<String, String>,
    pub price: i64,
}

impl From<Project<ObjectId>> for PublicProject {
    fn from(project: Project<ObjectId>) -> Self {
        Self {
            id: project.id.to_hex(),
            name: project.name,
            description: project.description,
            scope: project.scope,
            tags: project.tags,
            publish_options: project.publish_options,
            status: project.status,
            creator_contacts: project.creator_contacts,
            price: project.price,
        }
    }
}

impl From<Project<ObjectId>> for Option<Document> {
    fn from(project: Project<ObjectId>) -> Self {
        if !project.publish_options.publish {
            return None;
        }
        let mut document = bson::to_document(&project).unwrap();
        document.insert("kind", "project");
        Some(document)
    }
}
