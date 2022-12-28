use chrono::NaiveDateTime;
use mongodb::{Collection, bson::{oid::ObjectId}};
use crate::error::Result;

pub struct TokenModel {
    pub token: String,
    pub user_id: ObjectId,
    pub created: NaiveDateTime,
}

pub struct TokenRepository {
    inner: Collection<TokenModel>,
}

impl TokenRepository {
    pub fn new(uri: String) -> Self {
        todo!()
    }

    pub async fn create(&self, token: &TokenModel) ->  Result<TokenModel> {
        todo!()
    }

    pub async fn find(&self, token: &str) -> Result<Option<TokenModel>> {
        todo!()
    }

    pub async fn remove(&self, token: &str) -> Result<Option<TokenModel>> {
        todo!()
    }

    pub async fn find_by_user(&self, user_id: ObjectId) -> Result<Option<TokenModel>> {
        todo!()
    }

}