use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{
        badge::merge,
        user::AddLinkedAccount,
        linked_accounts::{LinkedService, GetXAccessToken, XAccessResponse, XUserResponse}
    },
    auth::Auth,
    context::GeneralContext,
    entities::user::{PublicUser, User, LinkedAccount},
    error::{self, AddCode},
};
use mongodb::bson::{oid::ObjectId, Bson};

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::{header, Client};
use std::env::var;

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

        if !Edit.get_access(&auth, &user) {
            users.insert(&user).await?;
            return Err(anyhow::anyhow!("User is not available to delete this user").code(403));
        }

        Ok(user.into())
    }

    pub async fn find_linked_account(&self, id: String, name: &LinkedService) -> error::Result<Option<User<ObjectId>>> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let users = users
            .find_many("linked_accounts.id", &Bson::String(id.clone()))
            .await?;

        let user = users.iter().cloned().find(|user| {
            if let Some(accounts) = &user.linked_accounts {
                accounts.iter().any(|account| {
                    account.id == id && account.name == *name
                })
            } else { false }
        });

        Ok(user)
    }

    pub async fn add_linked_account(
        &self,
        id: ObjectId,
        account: LinkedAccount,
        auth: Auth,
    ) -> error::Result<Option<User<ObjectId>>> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(mut user) = users.find("id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        if !Edit.get_access(&auth, &user) {
            return Err(anyhow::anyhow!("User is not available to change this user").code(403));
        }

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

    pub async fn create_linked_account(
        &self,
        id: ObjectId,
        data: AddLinkedAccount,
    ) -> error::Result<LinkedAccount> {
        let auth = self.context.auth();
        let client = Client::new();

        if data.service == LinkedService::X {
            let client_id = var("X_CLIENT_ID").unwrap();
            let client_secret = var("X_CLIENT_SECRET").unwrap();

            let x_auth = GetXAccessToken {
                code: data.code,
                client_id: client_id.clone(),
                grant_type: "authorization_code".to_string(),
                redirect_uri: "https://dev.auditdb.io/oauth/callback".to_string(),
                code_verifier: "challenge".to_string(),
            };

            let access_response = client
                .post("https://api.twitter.com/2/oauth2/token")
                .header(header::ACCEPT, "application/json")
                .header(header::CONTENT_TYPE, "application/json")
                .basic_auth(client_id, Some(client_secret))
                .json(&x_auth)
                .send()
                .await?
                .text()
                .await?;

            let access_json: XAccessResponse = serde_json::from_str(&access_response)?;
            let access_token = access_json.access_token;

            let user_response = client
                .get("https://api.twitter.com/2/users/me?user.fields=profile_image_url,url")
                .header(header::ACCEPT, "application/json")
                .bearer_auth(access_token)
                .send()
                .await?
                .text()
                .await?;

            let user_data: XUserResponse = serde_json::from_str(&user_response)?;

            let linked_account = LinkedAccount {
                id: user_data.data.id,
                name: LinkedService::X,
                email: "".to_string(),
                url: format!("https://twitter.com/{}", user_data.data.username),
                avatar: user_data.data.profile_image_url.unwrap_or_default(),
                is_public: false,
                username: user_data.data.username
            };

            Self::add_linked_account(&self, id, linked_account.clone(), auth).await?;

            return Ok(linked_account)
        }

        Err(anyhow::anyhow!("Error adding account").code(404))
    }
}
