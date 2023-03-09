use std::sync::Arc;

use common::{entities::user::User, repository::Repository};
use mongodb::bson::{oid::ObjectId, Bson};

#[derive(Clone)]
pub struct UserRepo(
    Arc<dyn Repository<User<ObjectId>, Error = mongodb::error::Error> + Send + Sync>,
);

impl UserRepo {
    pub fn new<T>(repo: T) -> Self
    where
        T: Repository<User<ObjectId>, Error = mongodb::error::Error> + Send + Sync + 'static,
    {
        Self(Arc::new(repo))
    }

    pub async fn create(&self, user: &User<ObjectId>) -> Result<bool, mongodb::error::Error> {
        self.0.create(user).await
    }

    pub async fn find(
        &self,
        id: ObjectId,
    ) -> Result<Option<User<ObjectId>>, mongodb::error::Error> {
        self.0.find("id", &Bson::ObjectId(id)).await
    }

    pub async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User<ObjectId>>, mongodb::error::Error> {
        self.0.find("email", &Bson::String(email.to_string())).await
    }

    pub async fn delete(
        &self,
        id: &ObjectId,
    ) -> Result<Option<User<ObjectId>>, mongodb::error::Error> {
        self.0.delete("id", id).await
    }

    pub async fn find_all(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<User<ObjectId>>, mongodb::error::Error> {
        self.0.find_all(skip, limit).await
    }
}
