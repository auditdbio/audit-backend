use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{badge::merge, user::validate_name},
    auth::Auth,
    context::GeneralContext,
    entities::user::{PublicUser, User, LinkedAccount},
    error::{self, AddCode},
};
use mongodb::bson::{oid::ObjectId, Bson};

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use super::auth::ChangePassword;

pub struct UserService {
    pub context: GeneralContext,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserChange {
    email: Option<String>,
    password: Option<String>,
    current_password: Option<String>,
    name: Option<String>,
    current_role: Option<String>,
    is_new: Option<bool>,
    link_id: Option<String>,
}

impl UserService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn create(
        &self,
        user: User<ObjectId>,
        merge_secret: Option<String>,
    ) -> error::Result<PublicUser> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        // run merge here
        if let Some(secret) = merge_secret {
            merge(&self.context, Auth::User(user.id), secret).await?;
        }

        users.insert(&user).await?;

        Ok(user.into())
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicUser>> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users.find("id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(&auth, &user) {
            return Err(anyhow::anyhow!("User is not available to read this user").code(403));
        }

        Ok(Some(user.into()))
    }

    pub async fn find_by_link_id(&self, link_id: String) -> error::Result<Option<PublicUser>> {
        let auth = self.context.auth();
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users.find("link_id", &Bson::String(link_id.clone())).await? else {
            return self.find(link_id.parse()?).await;
        };

        if !Read.get_access(&auth, &user) {
            return Err(anyhow::anyhow!("User is not available to read this user").code(403));
        }

        Ok(Some(user.into()))
    }

    pub async fn find_by_email(&self, email: String) -> error::Result<Option<User<ObjectId>>> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users.find("email", &email.into()).await? else {
            return Ok(None);
        };

        if !Read.get_access(&auth, &user) {
            return Err(anyhow::anyhow!("User is not available to read this user").code(403));
        }

        Ok(Some(user))
    }

    pub async fn find_linked_account(&self, id: i32, name: &str) -> error::Result<Option<User<ObjectId>>> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let users = users
            .find_many("linked_accounts.id", &Bson::Int32(id))
            .await?;

        let user = users.iter().cloned().find(|user| {
            if let Some(accounts) = &user.linked_accounts {
                accounts.iter().any(|account| {
                    account.id == id && account.name == name
                })
            } else { false }
        });

        Ok(user)
    }

    pub async fn add_linked_account(
        &self,
        id: ObjectId,
        account: LinkedAccount
    ) -> error::Result<Option<User<ObjectId>>> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(mut user) = users.find("id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        if let Some(ref mut linked_accounts) = user.linked_accounts {
            linked_accounts.push(account);
        } else {
            user.linked_accounts = Some(vec![account]);
        }

        user.last_modified = Utc::now().timestamp_micros();

        users.delete("id", &id).await?;
        users.insert(&user).await?;

        Ok(Some(user))
    }

    pub async fn my_user(&self) -> error::Result<Option<User<String>>> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users
            .find("id", &Bson::ObjectId(auth.id().unwrap()))
            .await?
        else {
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

        if !Edit.get_access(&auth, &user) {
            return Err(anyhow::anyhow!("User is not available to change this user").code(403));
        }

        if let Some(email) = change.email {
            user.email = email;
        }

        if let Some(mut password) = change.password {
            let Some(current_password) = change.current_password else {
                return Err(anyhow::anyhow!("Current password is required").code(400));
            };

            if !ChangePassword.get_access(current_password, &user) {
                return Err(anyhow::anyhow!("You wrote old password incorrectly.").code(403));
            }

            let salt: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect();

            password.push_str(&salt);
            let password = sha256::digest(password);
            user.password = password;
            user.salt = salt;
            user.is_passwordless = Some(false);
        }

        if let Some(name) = change.name {
            user.name = name.to_string();
        }

        if let Some(current_role) = change.current_role {
            user.current_role = current_role;
        }

        if let Some(is_new) = change.is_new {
            user.is_new = is_new;
        }

        if let Some(link_id) = change.link_id {
            if !validate_name(&link_id) {
                return Err(
                    anyhow::anyhow!("Username may only contain alphanumeric characters, hyphens or underscore")
                        .code(400)
                );
            }

            if users
                .find("link_id", &Bson::String(link_id.clone()))
                .await?
                .is_some() {
                return Err(anyhow::anyhow!("This link id is already taken").code(400));
            }

            if let Ok(parsed_id) = link_id.parse::<ObjectId>() {
                if let Some(user_by_id) = users
                    .find("id", &Bson::ObjectId(parsed_id))
                    .await? {
                    if user_by_id.id != id {
                        return Err(anyhow::anyhow!("This link id is already taken").code(400));
                    }
                }
            }

            user.link_id = link_id;
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

        if !Edit.get_access(&auth, &user) {
            users.insert(&user).await?;
            return Err(anyhow::anyhow!("User is not available to delete this user").code(403));
        }

        Ok(user.into())
    }
}
