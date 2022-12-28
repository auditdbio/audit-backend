use mongodb::{Collection, bson::oid::ObjectId};
use crate::error::Result;

pub struct UserModel {
    pub id: Option<ObjectId>,
    pub email: String,
    pub password: String,
    pub name: String,
}

pub struct UserRepository {
    inner: Collection<UserModel>,
}

impl UserRepository {
    pub fn new(uri: String) -> Self {
        todo!()
    }

    pub async fn create(&self, user: UserModel) ->  Result<Option<UserModel>> {
        todo!()
    }

    pub async fn find(&self, id: ObjectId) -> Result<Option<UserModel>> {
        todo!()
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserModel>> {
        todo!()
    }

    pub async fn delete(&self, id: ObjectId) -> Result<Option<UserModel>> {
        todo!()
    }

    pub async fn users(&self, page: u32, limit: u32) -> Result<Vec<UserModel>> {
        todo!()
    }
}