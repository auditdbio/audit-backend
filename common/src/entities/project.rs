use mongodb::bson::{self, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    context::Context,
    error,
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
    #[serde(default)]
    pub auditors: Vec<Id>,
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
            auditors: self
                .auditors
                .into_iter()
                .map(|id| id.parse().unwrap())
                .collect(),
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
            auditors: self.auditors.into_iter().map(|id| id.to_hex()).collect(),
        }
    }
}

impl Entity for Project<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicProject {
    pub id: String,
    pub customer_id: String,
    pub name: String,
    pub description: String,
    pub scope: Vec<String>,
    pub tags: Vec<String>,
    pub publish_options: PublishOptions,
    pub status: String,
    pub creator_contacts: Contacts,
    pub price: i64,
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
                "{}://{}/api/project/data",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str()
            ),
        );
        document.insert("kind", "project");
        document.insert("private", !project.publish_options.publish);
        Some(document)
    }
}

pub async fn get_project(context: &Context, id: ObjectId) -> error::Result<PublicProject> {
    Ok(context
        .make_request::<PublicProject>()
        .auth(context.server_auth()) // TODO: think about private projects here
        .get(format!(
            "{}://{}/api/project/{}",
            PROTOCOL.as_str(),
            CUSTOMERS_SERVICE.as_str(),
            id,
        ))
        .send()
        .await?
        .json::<PublicProject>()
        .await?)
}
