use std::collections::HashMap;

use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{
        customer::Customer,
        project::{Project, PublicProject, PublishOptions},
    },
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub description: String,
    pub scope: Vec<String>,
    pub tags: Vec<String>,
    pub publish_options: PublishOptions,
    pub status: String,
    pub price: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectChange {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scope: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub publish_options: Option<PublishOptions>,
    pub status: Option<String>,
    pub creator_contacts: Option<HashMap<String, String>>,
    pub price: Option<i64>,
}

pub struct ProjectService {
    context: Context,
}

impl ProjectService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, project: CreateProject) -> anyhow::Result<PublicProject> {
        let auth = self.context.auth();

        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };
        let customer = customers
            .find("user_id", &Bson::ObjectId(auth.id().unwrap().clone()))
            .await?
            .unwrap();

        let creator_contacts = if !project.publish_options.publish || customer.public_contacts {
            customer.contacts
        } else {
            HashMap::new()
        };

        let project = Project {
            id: ObjectId::new(),
            customer_id: auth
                .id()
                .ok_or(anyhow::anyhow!("No customer id found"))?
                .clone(),
            name: project.name,
            description: project.description,
            scope: project.scope,
            tags: project.tags,
            publish_options: project.publish_options,
            status: project.status,
            creator_contacts,
            price: project.price,
            last_modified: Utc::now().timestamp_micros(),
        };

        projects.insert(&project).await?;

        Ok(project.into())
    }

    pub async fn find(&self, id: ObjectId) -> anyhow::Result<Option<PublicProject>> {
        let auth = self.context.auth();

        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let Some(project) = projects.find("id",&Bson::ObjectId(id)).await? else {
            return Ok(None)
        };

        if !Read::get_access(auth, &project) {
            bail!("User is not available to read this project")
        }

        Ok(Some(project.into()))
    }

    pub async fn my_projects(&self) -> anyhow::Result<Vec<Project<String>>> {
        let auth = self.context.auth();

        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let projects = projects
            .find_many("customer_id", &Bson::ObjectId(auth.id().unwrap().clone()))
            .await?;

        Ok(projects.into_iter().map(Project::stringify).collect())
    }

    pub async fn change(
        &self,
        id: ObjectId,
        change: ProjectChange,
    ) -> anyhow::Result<PublicProject> {
        let auth = self.context.auth();

        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let Some(mut project) = projects.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No project found")
        };

        if !Edit::get_access(auth, &project) {
            bail!("User is not available to change this project")
        }

        if let Some(name) = change.name {
            project.name = name;
        }

        if let Some(description) = change.description {
            project.description = description;
        }

        if let Some(scope) = change.scope {
            project.scope = scope;
        }

        if let Some(tags) = change.tags {
            project.tags = tags;
        }

        if let Some(publish_options) = change.publish_options {
            project.publish_options = publish_options;
        }

        if let Some(status) = change.status {
            project.status = status;
        }

        if let Some(creator_contacts) = change.creator_contacts {
            project.creator_contacts = creator_contacts;
        }

        if let Some(price) = change.price {
            project.price = price;
        }

        project.last_modified = Utc::now().timestamp_micros();

        projects.delete("id", &id).await?;
        projects.insert(&project).await?;

        Ok(project.into())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicProject> {
        let auth = self.context.auth();

        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let Some(project) = projects.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No project found")
        };

        if !Edit::get_access(auth, &project) {
            projects.insert(&project).await?;
            bail!("User is not available to delete this project")
        }

        Ok(project.into())
    }
}
