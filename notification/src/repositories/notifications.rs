use common::repository::{mongo_repository::MongoRepository, Repository};
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId};

use crate::service::notifications::Notification;

pub struct NotificationsRepository {
    pub mongo: MongoRepository<Notification>,
}

impl NotificationsRepository {
    pub fn new(mongo: MongoRepository<Notification>) -> Self {
        Self { mongo }
    }

    pub async fn insert(&self, notification: &Notification) -> anyhow::Result<bool> {
        self.mongo.insert(notification).await
    }

    pub async fn get_unread(&self, user_id: &ObjectId) -> anyhow::Result<Vec<Notification>> {
        let res: Vec<mongodb::error::Result<Notification>> = self
            .mongo
            .collection
            .find(doc! {"inner.is_read": false, "user_id": user_id}, None)
            .await?
            .collect()
            .await;
        Ok(res
            .into_iter()
            .collect::<mongodb::error::Result<Vec<_>>>()?)
    }

    pub async fn read(&self, id: ObjectId) -> anyhow::Result<()> {
        self.mongo
            .collection
            .update_one(doc! {"id": id}, doc! {"$set": {"is_read": true}}, None)
            .await?;
        Ok(())
    }
}
