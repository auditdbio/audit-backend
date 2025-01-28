use chrono::Utc;
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

use common::{
    access_rules::{AccessRules, Edit, Read},
    api::seartch::{PaginationParams, delete_from_search},
    context::GeneralContext,
    entities::{
        contacts::Contacts,
        customer::{Customer, PublicCustomer},
        project::{Project, PublicProject, PublishOptions},
        scope::Scope,
    },
    error::{self, AddCode},
    services::{API_PREFIX, CUSTOMERS_SERVICE, PROTOCOL},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub description: String,
    pub scope: Scope,
    pub tags: Vec<String>,
    pub publish_options: PublishOptions,
    pub status: String,
    pub price: Option<i64>,
    pub total_cost: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectChange {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scope: Option<Scope>,
    pub tags: Option<Vec<String>>,
    pub publish_options: Option<PublishOptions>,
    pub status: Option<String>,
    pub price: Option<i64>,
    pub total_cost: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyProjectsResult {
    pub result: Vec<Project<String>>,
    #[serde(rename = "totalDocuments")]
    pub total_documents: u64,
}

pub struct ProjectService {
    context: GeneralContext,
}

impl ProjectService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn create(&self, project: CreateProject) -> error::Result<PublicProject> {
        let auth = self.context.auth();

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        let customer = customers
            .find("user_id", &Bson::ObjectId(auth.id().unwrap()))
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

        let price_per_line = if project.total_cost.is_none() {
            project.price
        } else {
            None
        };

        let project = Project {
            id: ObjectId::new(),
            customer_id: auth.id().ok_or(anyhow::anyhow!("No customer id found"))?,
            name: project.name,
            description: project.description,
            scope: project.scope,
            tags: project.tags,
            publish_options: project.publish_options,
            status: project.status,
            creator_contacts,
            price: price_per_line,
            total_cost: project.total_cost,
            last_modified: Utc::now().timestamp_micros(),
            created_at: Some(Utc::now().timestamp_micros()),
            auditors: Vec::new(),
        };

        projects.insert(&project).await?;

        Ok(auth.public_project(project))
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicProject>> {
        let auth = self.context.auth();

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let Some(mut project) = projects.find("id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(&auth, &project) {
            return Err(anyhow::anyhow!("User is not available to read this project").code(403));
        }

        let customer = self
            .context
            .make_request::<PublicCustomer>()
            .get(format!(
                "{}://{}/{}/customer/{}",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str(),
                API_PREFIX.as_str(),
                project.customer_id
            ))
            .auth(self.context.server_auth())
            .send()
            .await?
            .json::<PublicCustomer>()
            .await?;

        project.creator_contacts = customer.contacts;

        Ok(Some(auth.public_project(project)))
    }

    pub async fn my_projects(
        &self,
        pagination: PaginationParams,
    ) -> error::Result<Vec<Project<String>>> {
        let page = pagination.page.unwrap_or(0);
        let per_page = pagination.per_page.unwrap_or(0);
        let limit = pagination.per_page.unwrap_or(1000);
        let skip = (page - 1) * per_page;

        let auth = self.context.auth();

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let (projects, _total_documents) = projects
            .find_many_limit(
                "customer_id",
                &Bson::ObjectId(auth.id().unwrap()),
                skip,
                limit,
            )
            .await?;

        // Ok(MyProjectsResult {
        //     result: projects.into_iter().map(Project::stringify).collect(),
        //     total_documents,
        // })
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

        if !Edit.get_access(&auth, &project) {
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

        if change.total_cost.is_some() {
            project.total_cost = change.total_cost;
            project.price = None;
        }

        if change.price.is_some() && change.total_cost.is_none() {
            project.price = change.price;
            project.total_cost = None;
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

        if !Edit.get_access(&auth, &project) {
            projects.insert(&project).await?;
            return Err(anyhow::anyhow!("User is not available to delete this project").code(403));
        }

        delete_from_search(&self.context, id).await?;

        Ok(auth.public_project(project))
    }

    pub async fn add_auditor(
        &self,
        project_id: ObjectId,
        auditor_id: ObjectId,
    ) -> error::Result<()> {
        let auth = self.context.auth();
        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let Some(mut project) = projects.find("id", &Bson::ObjectId(project_id)).await? else {
            return Err(anyhow::anyhow!("No project found").code(404));
        };

        if !Edit.get_access(&auth, &project) {
            return Err(anyhow::anyhow!("User is not available to change this project").code(403));
        }

        project.auditors.push(auditor_id);

        projects.delete("id", &project_id).await?;
        projects.insert(&project).await?;

        Ok(())
    }

    pub async fn delete_auditor(
        &self,
        project_id: ObjectId,
        auditor_id: ObjectId,
    ) -> error::Result<()> {
        let auth = self.context.auth();
        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let Some(mut project) = projects.find("id", &Bson::ObjectId(project_id)).await? else {
            return Err(anyhow::anyhow!("No project found").code(404));
        };

        if !Edit.get_access(&auth, &project) {
            return Err(anyhow::anyhow!("User is not available to change this project").code(403));
        }

        project.auditors.retain(|id| id != &auditor_id);

        projects.delete("id", &project_id).await?;
        projects.insert(&project).await?;

        Ok(())
    }
}
