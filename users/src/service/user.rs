use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    auth::Auth,
    context::Context,
    entities::user::{PublicUser, User},
};
use mongodb::bson::{oid::ObjectId, Bson};

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

pub struct UserService {
    pub context: Context,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub name: String,
    pub current_role: String,
    pub use_email: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserChange {
    email: Option<String>,
    password: Option<String>,
    name: Option<String>,
    current_role: Option<String>,
}

impl UserService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, user: User<String>) -> anyhow::Result<PublicUser> {
        let auth = self.context.auth();

        // TODO: rewrite with get_access framework

        if let Auth::Service(_) = auth {
            bail!("Only services can create users")
        }

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let user = user.parse();

        users.insert(&user).await?;

        Ok(user.into())
    }

    pub async fn find(&self, id: ObjectId) -> anyhow::Result<Option<PublicUser>> {
        let auth = self.context.auth();

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(user) = users.find("id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read::get_access(auth, &user) {
            bail!("User is not available to read this user")
        }

        Ok(Some(user.into()))
    }

    pub async fn my_user(&self) -> anyhow::Result<Option<User<String>>> {
        let auth = self.context.auth();

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(user) = users.find("id", &Bson::ObjectId(auth.id().unwrap().clone())).await? else {
            return Ok(None);
        };

        Ok(Some(user.stringify()))
    }

    pub async fn change(&self, id: ObjectId, change: UserChange) -> anyhow::Result<PublicUser> {
        let auth = self.context.auth();

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(mut user) = users.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No user found")
        };

        if !Edit::get_access(auth, &user) {
            bail!("User is not available to change this user")
        }

        if let Some(email) = change.email {
            user.email = email;
        }

        if let Some(mut password) = change.password {
            let salt: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect();

            password.push_str(&salt);
            let password = sha256::digest(password);
            user.password = password;
            user.salt = salt;
        }

        if let Some(name) = change.name {
            user.name = name;
        }

        if let Some(current_role) = change.current_role {
            user.current_role = current_role;
        }

        user.last_modified = Utc::now().timestamp_micros();

        users.delete("id", &id).await?;
        users.insert(&user).await?;

        Ok(user.into())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicUser> {
        let auth = self.context.auth();

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(user) = users.delete("id", &id).await? else {
            bail!("No user found")
        };

        if !Edit::get_access(auth, &user) {
            users.insert(&user).await?;
            bail!("User is not available to delete this user")
        }

        Ok(user.into())
    }
}
