use std::collections::HashMap;

use mongodb::{Collection, bson::{oid::ObjectId, doc}, Client, error::Result as MongoResult};
use serde::{Serialize, Deserialize};

use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectModel {
    pub id: ObjectId,
    pub customer_id: ObjectId,
    pub name: String,
    pub description: String,
    pub git_url: String,
    pub git_folders: HashMap<String, String>,
    pub tags: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ProjectRepository {
    inner: Collection<ProjectModel>,
}

impl ProjectRepository {
    const DATABASE: &'static str = "Customers";
    const COLLECTION: &'static str = "Projects";

    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<ProjectModel> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, project: ProjectModel) -> Result<bool> {
        let exited_project = self.find(project.id).await?;

        if exited_project.is_some() {
            return Ok(false);
        }

        self.inner.insert_one(project, None).await?;
        Ok(true)
    }

    pub async fn find(&self, id: ObjectId) -> Result<Option<ProjectModel>> {
        Ok(self.inner.find_one(doc!{"id": id}, None).await?)
    }

    pub async fn delete(&self, id: ObjectId) -> Result<Option<ProjectModel>> {
        Ok(self.inner.find_one_and_delete(doc!{"id": id}, None).await?)
    }
}
