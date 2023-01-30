use common::entities::project::Project;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    error::Result as MongoResult,
    Client, Collection,
};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ProjectRepository {
    inner: Collection<Project>,
}

impl ProjectRepository {
    const DATABASE: &'static str = "Customers";
    const COLLECTION: &'static str = "Projects";

    #[allow(dead_code)] // is says that this function is not used, but it is used in main.rs
    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<Project> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, project: &Project) -> Result<bool> {
        self.inner.insert_one(project, None).await?;
        Ok(true)
    }

    pub async fn find(&self, id: &ObjectId) -> Result<Option<Project>> {
        Ok(self.inner.find_one(doc! {"id": id}, None).await?)
    }

    pub async fn delete(&self, id: ObjectId) -> Result<Option<Project>> {
        Ok(self
            .inner
            .find_one_and_delete(doc! {"id": id}, None)
            .await?)
    }

    pub async fn request_with_tags(&self, tags: Vec<String>) -> Result<Vec<Project>> {
        let filter = doc! {
            "tags": doc!{
                "$elemMatch": doc!{"$in": tags}
            }
        };

        let result: Vec<Project> = self
            .inner
            .find(filter, None)
            .await?
            .collect::<Vec<MongoResult<Project>>>()
            .await
            .into_iter()
            .collect::<MongoResult<_>>()?;
        Ok(result)
    }
}
