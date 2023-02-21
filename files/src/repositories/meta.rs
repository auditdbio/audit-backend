use chrono::NaiveDateTime;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client, Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaData {
    pub id: ObjectId,
    pub creator_id: ObjectId,
    pub last_modified: NaiveDateTime,
    pub content_type: String,
}

pub struct MetaRepository {
    inner: Collection<MetaData>,
}

impl MetaRepository {
    const DATABASE: &'static str = "Files";
    const COLLECTION: &'static str = "Metadata";

    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<MetaData> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, metadata: MetaData) -> Result<bool, mongodb::error::Error> {
        let exited_metadata = self.find(&metadata.id).await?;

        if exited_metadata.is_some() {
            return Ok(false);
        }

        self.inner.insert_one(metadata, None).await?;
        Ok(true)
    }

    pub async fn find(&self, id: &ObjectId) -> Result<Option<MetaData>, mongodb::error::Error> {
        Ok(self.inner.find_one(doc! {"id": id}, None).await?)
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<Option<MetaData>, mongodb::error::Error> {
        Ok(self
            .inner
            .find_one_and_delete(doc! {"id": id}, None)
            .await?)
    }
}
