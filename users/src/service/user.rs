use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Create, Edit, Read},
    context::Context,
    entities::user::User,
};
use mongodb::bson::{oid::ObjectId, Bson};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use super::auth::Code;

pub struct UserService {
    pub context: Context,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    email: String,
    password: String,
    name: String,
    code: String,
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
    email: Option<String>,
    password: Option<String>,
    name: Option<String>,
    current_role: Option<String>,
}

impl<'a, 'b> AccessRules<&'a CreateUser, &'b Code> for Create {
    fn get_access(user: &'a CreateUser, code: &'b Code) -> bool {
        &user.code == &code.code && &user.email == &code.email
    }
}

impl UserService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, mut user: CreateUser) -> anyhow::Result<PublicUser> {
        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(codes) = self.context.get_repository::<Code>() else {
            bail!("No code repository found")
        };

        let Some(code) = codes.find("email", &Bson::String(user.email.clone())).await? else {
            bail!("No code found")
        };

        if !Create::get_access(&user, &code) {
            bail!("User is not allowed to create this user")
        }

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
