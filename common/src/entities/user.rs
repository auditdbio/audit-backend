use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    ToSchema,
};

use crate::repository::Entity;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct User<Id> {
    pub id: Id,
    pub email: String,
    pub password: String,
    pub name: String,
    pub current_role: String,
}

impl User<String> {
    fn parse(self) -> User<ObjectId> {
        User {
            id: ObjectId::from_str(&self.id).unwrap(),
            email: self.email,
            password: self.password,
            name: self.name,
            current_role: self.current_role,
        }
    }
}

impl User<ObjectId> {
    pub fn stringify(self) -> User<String> {
        User {
            id: self.id.to_hex(),
            email: self.email,
            password: self.password,
            name: self.name,
            current_role: self.current_role,
        }
    }
}

impl Entity for User<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}
