use async_trait::async_trait;
use chrono::Utc;
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, to_document},
    options::FindOptions,
};
use mongodb::bson::Document;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{self, AddCode};
use crate::repository::HasLastModified;

use super::{Entity, Repository};

pub struct MongoRepository<T> {
    pub collection: mongodb::Collection<T>,
}

impl<T> MongoRepository<T> {
    pub async fn new(mongo_uri: &str, database: &str, collection: &str) -> Self {
        let collection = mongodb::Client::with_uri_str(mongo_uri)
            .await
            .unwrap()
            .database(database)
            .collection(collection);
        Self { collection }
    }
}

#[async_trait]
impl<T> Repository<T> for MongoRepository<T>
where
    T: Entity + Serialize + DeserializeOwned + Unpin + Clone + Send + Sync + HasLastModified,
{
    async fn insert(&self, item: &T) -> error::Result<bool> {
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

    async fn find(&self, field: &str, value: &Bson) -> error::Result<Option<T>> {
        let result = self.collection.find_one(doc! {field: value}, None).await?;
        Ok(result)
    }

    async fn delete(&self, field: &str, item: &ObjectId) -> error::Result<Option<T>> {
        let result = self
            .collection
            .find_one_and_delete(doc! {field: item}, None)
            .await?;
        Ok(result)
    }

    async fn update_one(&self, mut old: Document, update: &T) -> error::Result<bool> {
        // old.insert("last_modified", Bson::Int64(update.last_modified()));
        old.extend(doc! {
            "$or": [
                { "last_modified": Bson::Int64(update.last_modified()) },
                { "last_modified": { "$exists": false } }
            ]
        });

        let mut update = update.clone();
        update.set_last_modified(Utc::now().timestamp_micros());

        let result = self
            .collection
            .find_one_and_update(old, doc! {"$set": to_document(&update)?}, None)
            .await?
            .is_some();

        if !result {
            return Err(anyhow::anyhow!("Failed to save changes").code(409));
        }

        Ok(result)
    }

    async fn find_all(&self, skip: u32, limit: u32) -> error::Result<Vec<T>> {
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

    async fn find_many(&self, field: &str, value: &Bson) -> error::Result<Vec<T>> {
        let result: Vec<mongodb::error::Result<T>> = self
            .collection
            .find(doc! {field: value}, None)
            .await?
            .collect()
            .await;
        Ok(result.into_iter().collect::<mongodb::error::Result<_>>()?)
    }

    async fn find_many_limit(
        &self,
        field: &str,
        value: &Bson,
        skip: i32,
        limit: i32,
    ) -> error::Result<(Vec<T>, u64)> {
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(limit as i64)
            .build();

        let result: Vec<mongodb::error::Result<T>> = self
            .collection
            .find(doc! {field: value}, find_options)
            .await?
            .collect()
            .await;

        let total_documents = self
            .collection.count_documents(doc! {field: value}, None)
            .await?;

        Ok((result.into_iter().collect::<mongodb::error::Result<_>>()?, total_documents))
    }

    async fn get_all_since(&self, since: i64) -> error::Result<Vec<T>> {
        let result: Vec<mongodb::error::Result<T>> = self
            .collection
            .find(doc! {"last_modified": doc! {"$gt": since}}, None)
            .await?
            .collect()
            .await;
        Ok(result.into_iter().collect::<mongodb::error::Result<_>>()?)
    }

    async fn find_all_by_ids(&self, id: &str, ids: Vec<ObjectId>) -> error::Result<Vec<T>> {
        let result: Vec<mongodb::error::Result<T>> = self
            .collection
            .find(doc! {id: doc! {"$in": ids}}, None)
            .await?
            .collect()
            .await;
        Ok(result.into_iter().collect::<mongodb::error::Result<_>>()?)
    }
}
