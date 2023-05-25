use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::user::{PublicUser, User}, error::{AddCode, self},
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
    is_new: Option<bool>,
}

impl UserService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, user: User<String>) -> error::Result<PublicUser> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;
        let user = user.parse();

        users.insert(&user).await?;

        Ok(user.into())
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicUser>> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users.find("id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(auth, &user) {
            return Err(anyhow::anyhow!("User is not available to read this user").code(403));
        }

        Ok(Some(user.into()))
    }

    pub async fn my_user(&self) -> error::Result<Option<User<String>>> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users.find("id", &Bson::ObjectId(auth.id().unwrap().clone())).await? else {
            return Ok(None);
        };

        Ok(Some(user.stringify()))
    }

    pub async fn change(&self, id: ObjectId, change: UserChange) -> error::Result<PublicUser> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(mut user) = users.find("id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        if !Edit.get_access(auth, &user) {
            return Err(anyhow::anyhow!("User is not available to change this user").code(403));
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

        if let Some(is_new) = change.is_new {
            user.is_new = is_new;
        }

        user.last_modified = Utc::now().timestamp_micros();

        users.delete("id", &id).await?;
        users.insert(&user).await?;

        Ok(user.into())
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<PublicUser> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users.delete("id", &id).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        if !Edit.get_access(auth, &user) {
            users.insert(&user).await?;
            return Err(anyhow::anyhow!("User is not available to delete this user").code(403));
        }

        Ok(user.into())
    }
}
