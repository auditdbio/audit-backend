use anyhow::bail;
use chrono::Utc;
use common::{context::Context, entities::project::Project};
use mongodb::bson::{oid::ObjectId, Bson};

pub struct CreateProject {}

pub struct ProjectChange {}

pub struct PublicProject {}

pub struct ProjectService {
    context: Context,
}

impl ProjectService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, project: CreateProject) -> anyhow::Result<PublicProject> {
        let auth = self.context.auth_res()?;

        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let project = Project {};
    }

    pub async fn find(&self, id: ObjectId) -> anyhow::Result<Option<PublicProject>> {
        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let project = projects.find(id).await?;

        Ok(project.into())
    }

    pub async fn change(&self, change: ProjectChange) -> anyhow::Result<PublicProject> {
        let auth = self.context.auth_res()?;

        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let Some(mut project) = projects.find("id", &Bson::ObjectId(change.id)).await? else {
            bail!("No project found")
        };

        if !Edit::get_access(&auth, &project) {
            bail!("User is not available to change this project")
        }

        if let Some(avatar) = change.avatar {
            project.avatar = avatar;
        }

        if let Some(first_name) = change.first_name {
            project.first_name = first_name;
        }

        if let Some(last_name) = change.last_name {
            project.last_name = last_name;
        }

        if let Some(about) = change.about {
            project.about = about;
        }

        if let Some(company) = change.company {
            project.company = company;
        }

        if let Some(contacts) = change.contacts {
            project.contacts = contacts;
        }

        if let Some(tags) = change.tags {
            project.tags = tags;
        }

        project.last_modified = Utc::now().timestamp_micros();

        projects.delete("id", &change.id).await?;
        projects.insert(&project).await?;

        Ok(project.into())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicProject> {
        let auth = self.context.auth_res()?;

        let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
            bail!("No project repository found")
        };

        let Some(project) = projects.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No project found")
        };

        if !Edit::get_access(&auth, &project) {
            bail!("User is not available to delete this project")
        }

        Ok(project.into())
    }
}
