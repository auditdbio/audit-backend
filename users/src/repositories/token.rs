use chrono::NaiveDateTime;
use mongodb::{Collection, bson::{oid::ObjectId, doc}, Client};
use serde::{Serialize, Deserialize};
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenModel {
    pub token: String,
    pub user_id: ObjectId,
    pub created: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct TokenRepository {
    inner: Collection<TokenModel>,
}

impl TokenRepository {
    const DATABASE: &'static str = "Users";
    const COLLECTION: &'static str = "Tokens";


    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<TokenModel> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, token: &TokenModel) ->  Result<()> {
        self.inner.insert_one(token, None).await?;
        Ok(())
    }

    pub async fn find(&self, token: &str) -> Result<Option<TokenModel>> {
        Ok(self.inner.find_one(doc!{"token": token}, None).await?)
    }

    pub async fn delete(&self, token: &str) -> Result<Option<TokenModel>> {
        Ok(self.inner.find_one_and_delete(doc!{"token": token}, None).await?)
    }

    pub async fn find_by_user(&self, user_id: ObjectId) -> Result<Option<TokenModel>> {
        Ok(self.inner.find_one(doc!{"user_id": user_id}, None).await?)
    }
}
