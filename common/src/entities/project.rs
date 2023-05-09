use mongodb::bson::{self, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    repository::Entity,
    services::{CUSTOMERS_SERVICE, PROTOCOL},
};

use super::contacts::Contacts;

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
    pub creator_contacts: Contacts,
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
    pub creator_contacts: Contacts,
    pub price: i64,
}

impl From<Project<ObjectId>> for PublicProject {
    fn from(project: Project<ObjectId>) -> Self {
        let creator_contacts = if project.creator_contacts.public_contacts {
            project.creator_contacts
        } else {
            Contacts {
                email: None,
                telegram: None,
                public_contacts: false,
            }
        };

        Self {
            id: project.id.to_hex(),
            name: project.name,
            description: project.description,
            scope: project.scope,
            tags: project.tags,
            publish_options: project.publish_options,
            status: project.status,
            creator_contacts,
            price: project.price,
        }
    }
}

impl From<Project<ObjectId>> for Option<Document> {
    fn from(project: Project<ObjectId>) -> Self {
        let project = project.stringify();
        let mut document = bson::to_document(&project).unwrap();
        document.insert(
            "search_tags",
            project
                .tags
                .iter()
                .map(|tag| tag.to_lowercase())
                .collect::<Vec<String>>(),
        );
        document.insert(
            "request_url",
            format!(
                "{}://{}/project/data",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str()
            ),
        );
        document.insert("kind", "project");
        document.insert("private", !project.publish_options.publish);
        Some(document)
    }
}
