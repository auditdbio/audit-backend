use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::Entity;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct User<Id> {
    pub id: Id,
    pub email: String,
    pub password: String,
    pub salt: String,
    pub name: String,
    pub current_role: String,
    pub last_modified: i64,
}

impl User<String> {
    pub fn parse(self) -> User<ObjectId> {
        User {
            id: self.id.parse().unwrap(),
            email: self.email,
            password: self.password,
            salt: self.salt,
            name: self.name,
            current_role: self.current_role,
            last_modified: self.last_modified,
        }
    }
}

impl User<ObjectId> {
    pub fn stringify(self) -> User<String> {
        User {
            id: self.id.to_hex(),
            email: self.email,
            password: self.password,
            salt: self.salt,
            name: self.name,
            current_role: self.current_role,
            last_modified: self.last_modified,
        }
    }
}

impl Entity for User<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicUser {
    id: String,
    pub email: String,
    pub name: String,
    current_role: String,
}

impl From<User<ObjectId>> for PublicUser {
    fn from(user: User<ObjectId>) -> Self {
        Self {
            id: user.id.to_hex(),
            email: user.email,
            name: user.name,
            current_role: user.current_role,
        }
    }
}
