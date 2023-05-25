use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{
        contacts::Contacts,
        customer::Customer,
        project::{Project, PublicProject, PublishOptions},
    }, error::{AddCode, self},
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
    pub price: Option<i64>,
}

pub struct ProjectService {
    context: Context,
}

impl ProjectService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, project: CreateProject) -> error::Result<PublicProject> {
        let auth = self.context.auth();

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        let customer = customers
            .find("user_id", &Bson::ObjectId(auth.id().unwrap().clone()))
            .await?
            .unwrap();

        let creator_contacts =
            if !project.publish_options.publish || customer.contacts.public_contacts {
                customer.contacts
            } else {
                Contacts {
                    email: None,
                    telegram: None,
                    public_contacts: false,
                }
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

        Ok(auth.public_project(project))
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicProject>> {
        let auth = self.context.auth();

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let Some(project) = projects.find("id",&Bson::ObjectId(id)).await? else {
            return Ok(None)
        };

        if !Read.get_access(auth, &project) {
            return Err(anyhow::anyhow!("User is not available to read this project").code(403));
        }

        Ok(Some(auth.public_project(project)))
    }

    pub async fn my_projects(&self) -> error::Result<Vec<Project<String>>> {
        let auth = self.context.auth();

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let projects = projects
            .find_many("customer_id", &Bson::ObjectId(auth.id().unwrap().clone()))
            .await?;

        Ok(projects.into_iter().map(Project::stringify).collect())
    }

    pub async fn change(
        &self,
        id: ObjectId,
        change: ProjectChange,
    ) -> error::Result<PublicProject> {
        let auth = self.context.auth();

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let Some(mut project) = projects.find("id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No project found").code(404));
        };

        if !Edit.get_access(auth, &project) {
            return Err(anyhow::anyhow!("User is not available to change this project").code(403));
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

        if let Some(price) = change.price {
            project.price = price;
        }

        project.last_modified = Utc::now().timestamp_micros();

        projects.delete("id", &id).await?;
        projects.insert(&project).await?;

        Ok(auth.public_project(project))
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<PublicProject> {
        let auth = self.context.auth();

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let Some(project) = projects.delete("id", &id).await? else {
            return Err(anyhow::anyhow!("No project found").code(404));
        };

        if !Edit.get_access(auth, &project) {
            projects.insert(&project).await?;
            return Err(anyhow::anyhow!("User is not available to delete this project").code(403));
        }

        Ok(auth.public_project(project))
    }
}
