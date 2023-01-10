use mongodb::{Collection, bson::{oid::ObjectId, doc}, Client, options::FindOptions};
use serde::{Serialize, Deserialize};
use crate::error::Result;
use futures::stream::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModel {
    pub id: ObjectId,
    pub email: String,
    pub password: String,
    pub name: String,
}

impl UserModel {
    pub fn new(email: String, password: String, name: String) -> Self {
        Self {
            id: ObjectId::new(),
            email,
            password,
            name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRepository {
    inner: Collection<UserModel>,
}

impl UserRepository {
    const DATABASE: &'static str = "Users";
    const COLLECTION: &'static str = "Users";

    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<UserModel> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, user: &UserModel) ->  Result<bool> {
        let user_with_same_email = self.find_by_email(&user.email).await?;

        if user_with_same_email.is_some() {
            return Ok(false);
        }

        self.inner.insert_one(user, None).await?;
        Ok(true)
    }

    pub async fn find(&self, id: ObjectId) -> Result<Option<UserModel>> {
        Ok(self.inner.find_one(doc!{"id": id}, None).await?)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserModel>> {
        Ok(self.inner.find_one(doc!{"email": email}, None).await?)
    }

    pub async fn delete(&self, id: ObjectId) -> Result<Option<UserModel>> {
        Ok(self.inner.find_one_and_delete(doc!{"id": id}, None).await?)
    }

    pub async fn users(&self, skip: u32, limit: u32) -> Result<Vec<UserModel>> {
        let find_options = FindOptions::builder()
            .skip(skip as u64).limit(limit as i64)
            .build();
        
        let users: Vec<mongodb::error::Result<UserModel>> = self
            .inner
            .find(None, find_options)
            .await?
            .collect()
            .await;
        
        Ok(users.into_iter().collect::<mongodb::error::Result<_>>()?)
    }
}
