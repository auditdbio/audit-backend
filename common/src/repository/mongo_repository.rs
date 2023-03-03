use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    options::FindOptions,
};
use serde::{de::DeserializeOwned, Serialize};

use super::{Entity, Repository, TaggableEntityRepository};

pub struct MongoRepository<T> {
    pub collection: mongodb::Collection<T>,
}

impl<T> MongoRepository<T> {
    pub async fn new(mongo_uri: &str, database: &str, collection: &str) -> Self {
        Self {
            collection: mongodb::Client::with_uri_str(mongo_uri)
                .await
                .unwrap()
                .database(database)
                .collection(collection),
        }
    }
}

#[async_trait]
impl<T> Repository<T> for MongoRepository<T>
where
    T: Entity + Serialize + DeserializeOwned + Unpin + Clone + Send + Sync,
{
    type Error = mongodb::error::Error;

    async fn create(&self, item: &T) -> Result<bool, Self::Error> {
        let result = self
            .collection
            .find_one(doc! {"id": item.id()}, None)
            .await?
            .is_none();
        
        if result {
            self.collection.insert_one(item, None).await?;
        }
        Ok(result)
    }

    async fn find(&self, field: &str, value: &Bson) -> Result<Option<T>, Self::Error> {
        let result = self.collection.find_one(doc! {field: value}, None).await?;
        Ok(result)
    }

    async fn delete(&self, field: &str, item: &ObjectId) -> Result<Option<T>, Self::Error> {
        let result = self
            .collection
            .find_one_and_delete(doc! {field: item}, None)
            .await?;
        Ok(result)
    }

    async fn find_all(&self, skip: u32, limit: u32) -> Result<Vec<T>, Self::Error> {
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(limit as i64)
            .build();

        let results: Vec<mongodb::error::Result<T>> = self
            .collection
            .find(None, find_options)
            .await?
            .collect()
            .await;

        Ok(results.into_iter().collect::<mongodb::error::Result<_>>()?)
    }

    async fn find_many(&self, field: &str, value: &Bson) -> Result<Vec<T>, Self::Error> {
        let result: Vec<mongodb::error::Result<T>> = self
            .collection
            .find(doc! {field: value}, None)
            .await?
            .collect()
            .await;
        Ok(result.into_iter().collect::<mongodb::error::Result<_>>()?)
    }
}

#[async_trait]
impl<T> TaggableEntityRepository<T> for MongoRepository<T>
where
    T: Entity + Serialize + DeserializeOwned + Unpin + Clone + Send + Sync,
{
    async fn find_by_tags(&self, tags: Vec<String>) -> Result<Vec<T>, Self::Error> {
        use mongodb::error::Result as MongoResult;
        let filter = doc! {
            "tags": doc!{
                "$elemMatch": doc!{"$in": tags}
            }
        };

        let result: Vec<T> = self
            .collection
            .find(filter, None)
            .await?
            .collect::<Vec<MongoResult<T>>>()
            .await
            .into_iter()
            .collect::<MongoResult<_>>()?;
        Ok(result)
    }
}