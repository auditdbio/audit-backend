use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::repository::Entity;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkedAccount {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub url: String,
    pub avatar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User<Id> {
    pub id: Id,
    pub email: String,
    pub password: String,
    pub salt: String,
    pub name: String,
    pub current_role: String,
    pub last_modified: i64,
    pub created_at: Option<i64>,
    pub is_new: bool,
    #[serde(default)]
    pub is_admin: bool,
    pub linked_accounts: Option<Vec<LinkedAccount>>,
    pub is_passwordless: Option<bool>,
    #[serde(default)]
    pub link_id: String,
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
            created_at: self.created_at,
            is_new: self.is_new,
            is_admin: self.is_admin,
            linked_accounts: self.linked_accounts,
            is_passwordless: self.is_passwordless,
            link_id: self.link_id,
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
            created_at: self.created_at,
            is_new: self.is_new,
            is_admin: self.is_admin,
            linked_accounts: self.linked_accounts,
            is_passwordless: self.is_passwordless,
            link_id: self.link_id,
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
    pub id: String,
    pub email: String,
    pub name: String,
    pub current_role: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLogin {
    pub id: String,
    pub email: String,
    pub name: String,
    pub current_role: String,
    pub last_modified: i64,
    pub created_at: Option<i64>,
    pub is_new: bool,
    pub is_admin: bool,
    pub linked_accounts: Option<Vec<LinkedAccount>>,
    pub is_passwordless: Option<bool>,
}

impl From<User<ObjectId>> for UserLogin {
    fn from(user: User<ObjectId>) -> Self {
        Self {
            id: user.id.to_hex(),
            email: user.email,
            name: user.name,
            current_role: user.current_role,
            last_modified: user.last_modified,
            created_at: user.created_at,
            is_new: user.is_new,
            is_admin: user.is_admin,
            linked_accounts: user.linked_accounts,
            is_passwordless: user.is_passwordless,
        }
    }
}
