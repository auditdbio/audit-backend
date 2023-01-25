use crate::error::Result;
use common::entities::user::User;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::FindOptions,
    Client, Collection,
};

#[derive(Debug, Clone)]
pub struct UserRepository {
    inner: Collection<User>,
}

impl UserRepository {
    const DATABASE: &'static str = "Users";
    const COLLECTION: &'static str = "Users";

    #[allow(dead_code)] // is says that this function is not used, but it is used in main.rs
    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<User> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, user: &User) -> Result<bool> {
        let user_with_same_email = self.find_by_email(&user.email).await?;

        if user_with_same_email.is_some() {
            return Ok(false);
        }

        self.inner.insert_one(user, None).await?;
        Ok(true)
    }

    pub async fn find(&self, id: ObjectId) -> Result<Option<User>> {
        Ok(self.inner.find_one(doc! {"id": id}, None).await?)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        Ok(self.inner.find_one(doc! {"email": email}, None).await?)
    }

    pub async fn delete(&self, id: ObjectId) -> Result<Option<User>> {
        Ok(self
            .inner
            .find_one_and_delete(doc! {"id": id}, None)
            .await?)
    }

    pub async fn users(&self, skip: u32, limit: u32) -> Result<Vec<User>> {
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(limit as i64)
            .build();

        let users: Vec<mongodb::error::Result<User>> =
            self.inner.find(None, find_options).await?.collect().await;

        Ok(users.into_iter().collect::<mongodb::error::Result<_>>()?)
    }
}
