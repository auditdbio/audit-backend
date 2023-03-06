use common::repository::{Entity, Repository};
use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenModel {
    pub token: String,
    pub user_id: ObjectId,
    pub exp: usize,
}

impl Entity for TokenModel {
    fn id(&self) -> ObjectId {
        self.user_id.clone()
    }
}

#[derive(Clone)]
pub struct TokenRepo(Arc<dyn Repository<TokenModel, Error = mongodb::error::Error> + Send + Sync>);

impl TokenRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<TokenModel, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }
    pub async fn create(&self, user: &TokenModel) -> Result<bool, mongodb::error::Error> {
        self.0.create(user).await
    }

    pub async fn find_by_token(
        &self,
        token: String,
    ) -> Result<Option<TokenModel>, mongodb::error::Error> {
        self.0.find("token", &Bson::String(token)).await
    }

    pub async fn delete(&self, token: String) -> Result<Option<TokenModel>, mongodb::error::Error> {
        let token = self.find_by_token(token).await?;
        if let Some(token) = token {
            self.0.delete("id", &token.user_id).await
        } else {
            Ok(None)
        }
    }
}
