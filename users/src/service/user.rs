use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::user::User,
};
use mongodb::bson::{oid::ObjectId, Bson};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

pub struct UserService {
    pub context: Context,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    email: String,
    password: String,
    name: String,
    current_role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicUser {
    id: String,
    email: String,
    name: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UserChange {
    id: ObjectId,
    email: Option<String>,
    password: Option<String>,
    name: Option<String>,
    current_role: Option<String>,
}

impl UserService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create_user(&self, mut user: CreateUser) -> anyhow::Result<PublicUser> {
        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let salt: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        user.password.push_str(&salt);
        let password = sha256::digest(user.password);

        let user = User {
            id: ObjectId::new(),
            name: user.name,
            email: user.email,
            salt,
            password,
            current_role: user.current_role,
            last_modified: Utc::now().timestamp_micros(),
        };

        if users
            .find("email", &Bson::String(user.email.clone()))
            .await?
            .is_some()
        {
            bail!("User with email already exists")
        }

        users.insert(&user).await?;

        Ok(user.into())
    }

    pub async fn find_user(&self, id: ObjectId) -> anyhow::Result<PublicUser> {
        let Some(auth) = &self.context.1.user_auth else {
            bail!("No user auth found")
        };

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(user) = users.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No user found")
        };

        if !Read::get_access(auth, &user) {
            bail!("User is not available to read this user")
        }

        Ok(user.into())
    }

    pub async fn change_user(&self, change: UserChange) -> anyhow::Result<PublicUser> {
        let Some(auth) = &self.context.1.user_auth else {
            bail!("No user auth found")
        };

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(mut user) = users.find("id", &Bson::ObjectId(change.id)).await? else {
            bail!("No user found")
        };

        if !Edit::get_access(auth, &user) {
            bail!("User is not available to change this user")
        }

        if let Some(email) = change.email {
            user.email = email;
        }

        if let Some(password) = change.password {
            user.password = password;
        }

        if let Some(name) = change.name {
            user.name = name;
        }

        if let Some(current_role) = change.current_role {
            user.current_role = current_role;
        }

        user.last_modified = Utc::now().timestamp_micros();

        users.delete("id", &change.id).await?;
        users.insert(&user).await?;

        Ok(user.into())
    }

    pub async fn delete_user(&self, id: ObjectId) -> anyhow::Result<PublicUser> {
        let Some(auth) = &self.context.1.user_auth else {
            bail!("No user auth found")
        };

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(user) = users.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No user found")
        };

        if !Edit::get_access(auth, &user) {
            bail!("User is not available to delete this user")
        }

        Ok(user.into())
    }
}
