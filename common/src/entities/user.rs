use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::{
    api::linked_accounts::LinkedService,
    repository::Entity,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkedAccount {
    pub id: String,
    pub name: LinkedService,
    pub email: String,
    pub url: String,
    pub avatar: String,
    #[serde(default)]
    pub is_public: bool,
    #[serde(default)]
    pub username: String,
    pub token: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicLinkedAccount {
    pub id: String,
    pub name: LinkedService,
    pub username: String,
    pub email: String,
    pub url: String,
    pub avatar: String,
    pub is_public: bool,
}

impl From<LinkedAccount> for PublicLinkedAccount {
    fn from(account: LinkedAccount) -> Self {
        Self {
            id: account.id,
            name: account.name,
            username: account.username,
            email: account.email,
            url: account.url,
            avatar: account.avatar,
            is_public: account.is_public,
        }
    }
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
    pub linked_accounts: Option<Vec<PublicLinkedAccount>>,
    pub link_id: String,
}

impl From<User<ObjectId>> for PublicUser {
    fn from(user: User<ObjectId>) -> Self {
        let linked_accounts = user.linked_accounts.map(|acc| {
            acc
                .into_iter()
                .map(PublicLinkedAccount::from)
                .filter(|acc| acc.is_public)
                .collect()
        });

        Self {
            id: user.id.to_hex(),
            email: user.email,
            name: user.name,
            current_role: user.current_role,
            link_id: user.link_id,
            linked_accounts,
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
    pub linked_accounts: Option<Vec<PublicLinkedAccount>>,
    pub is_passwordless: Option<bool>,
    pub link_id: String,
}

impl From<User<ObjectId>> for UserLogin {
    fn from(user: User<ObjectId>) -> Self {
        let accounts = user.linked_accounts.map(|acc| {
            acc.into_iter().map(PublicLinkedAccount::from).collect()
        });

        Self {
            id: user.id.to_hex(),
            email: user.email,
            name: user.name,
            current_role: user.current_role,
            last_modified: user.last_modified,
            created_at: user.created_at,
            is_new: user.is_new,
            is_admin: user.is_admin,
            linked_accounts: accounts,
            is_passwordless: user.is_passwordless,
            link_id: user.link_id,
        }
    }
}
