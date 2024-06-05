use chrono::Utc;
use mongodb::bson::{oid::ObjectId, Bson};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use reqwest::{header, Client, StatusCode};
use std::env::var;
use actix_web::HttpResponse;
use web3::signing::{keccak256, recover};

use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{
        badge::merge,
        linked_accounts::{
            AddLinkedAccount, LinkedService,
            GetXAccessToken, XAccessResponse,
            XUserResponse, GetLinkedInAccessToken,
            LinkedInAccessResponse, LinkedInUserResponse,
            GetGithubAccessToken, UpdateLinkedAccount,
            AddWallet,
        },
        user::{get_access_token, create_request_with_token},
    },
    auth::Auth,
    context::GeneralContext,
    entities::{
        user::{PublicUser, User, LinkedAccount, PublicLinkedAccount, UserLogin},
    },
    error::{self, AddCode},
    services::{PROTOCOL, USERS_SERVICE, API_PREFIX},
};

use crate::service::auth::AuthService;

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

    pub async fn my_user(&self) -> error::Result<Option<UserLogin>> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users
            .find("id", &Bson::ObjectId(auth.id().unwrap()))
            .await?
        else {
            return Ok(None);
        };

        Ok(Some(UserLogin::from(user)))
    }

    pub async fn change(&self, id: ObjectId, change: UserChange) -> error::Result<UserLogin> {
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

    pub async fn find_user_by_linked_account(
        &self,
        account_id: String,
        name: &LinkedService
    ) -> error::Result<Option<User<ObjectId>>> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let users = users
            .find_many("linked_accounts.id", &Bson::String(account_id.clone()))
            .await?;

        let user = users.iter().cloned().find(|user| {
            if let Some(accounts) = &user.linked_accounts {
                accounts.iter().any(|account| {
                    account.id == account_id && account.name == *name
                })
            } else { false }
        });

        Ok(user)
    }

    pub async fn add_linked_account(
        &self,
        user_id: ObjectId,
        account: LinkedAccount,
        auth: Auth,
    ) -> error::Result<Option<User<ObjectId>>> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(mut user) = users.find("id", &Bson::ObjectId(user_id)).await? else {
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

        users.delete("id", &user_id).await?;
        users.insert(&user).await?;

        Ok(Some(user))
    }

    pub async fn create_linked_account(
        &self,
        id: ObjectId,
        data: AddLinkedAccount,
    ) -> error::Result<PublicLinkedAccount> {
        let auth = self.context.auth();
        let client = Client::new();
        let protocol = var("PROTOCOL").unwrap();
        let frontend = var("FRONTEND").unwrap();

        if data.service == LinkedService::GitHub {
            let github_auth = GetGithubAccessToken {
                code: data.clone().code,
                client_id: var("GITHUB_CLIENT_ID").unwrap(),
                client_secret: var("GITHUB_CLIENT_SECRET").unwrap(),
            };

            let (_, linked_account) = AuthService::new(self.context.clone())
                .github_get_user(github_auth, data.clone().current_role).await?;

            if Self::find_user_by_linked_account(
                &self,
                linked_account.id.clone(),
                &LinkedService::GitHub
            ).await?.is_some() {
                if let Some(update_token) = data.update_token {
                    if update_token {
                        let _ = self.context
                            .make_request()
                            .patch(format!(
                                "{}://{}/{}/user/{}/linked_account/{}",
                                PROTOCOL.as_str(),
                                USERS_SERVICE.as_str(),
                                API_PREFIX.as_str(),
                                id,
                                linked_account.id.clone(),
                            ))
                            .auth(self.context.server_auth())
                            .json(&UpdateLinkedAccount {
                                is_public: None,
                                update_token: Some(true),
                                token: linked_account.token,
                                scope: linked_account.scope,
                            })
                            .send()
                            .await
                            .unwrap();
                    }
                }

                return Err(anyhow::anyhow!("Account has already been added").code(404))
            }

            Self::add_linked_account(&self, id, linked_account.clone(), auth).await?;

            return Ok(PublicLinkedAccount::from(linked_account))
        }

        if data.service == LinkedService::X {
            let client_id = var("X_CLIENT_ID").unwrap();
            let client_secret = var("X_CLIENT_SECRET").unwrap();

            let x_auth = GetXAccessToken {
                code: data.code,
                client_id: client_id.clone(),
                grant_type: "authorization_code".to_string(),
                redirect_uri: format!("{}://{}/oauth/callback", protocol, frontend),
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

            let account_data: XUserResponse = serde_json::from_str(&user_response)?;

            let linked_account = LinkedAccount {
                id: account_data.data.id,
                name: LinkedService::X,
                email: "".to_string(),
                url: format!("https://twitter.com/{}", account_data.data.username),
                avatar: account_data.data.profile_image_url.unwrap_or_default(),
                is_public: false,
                username: account_data.data.username,
                token: None,
                scope: None,
            };

            if Self::find_user_by_linked_account(
                &self,
                linked_account.id.clone(),
                &LinkedService::X
            ).await?.is_some() {
                return Err(anyhow::anyhow!("Account has already been added").code(404))
            }

            Self::add_linked_account(&self, id, linked_account.clone(), auth).await?;

            return Ok(PublicLinkedAccount::from(linked_account))
        }

        if data.service == LinkedService::LinkedIn {
            let client_id = var("LINKEDIN_CLIENT_ID").unwrap();
            let client_secret = var("LINKEDIN_CLIENT_SECRET").unwrap();

            let linkedin_auth = GetLinkedInAccessToken {
                code: data.code,
                client_id: client_id.clone(),
                client_secret,
                grant_type: "authorization_code".to_string(),
                redirect_uri: format!("{}://{}/oauth/callback", protocol, frontend),
            };

            let access_response = client
                .post("https://www.linkedin.com/oauth/v2/accessToken")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .form(&linkedin_auth)
                .send()
                .await?
                .text()
                .await?;

            let access_json: LinkedInAccessResponse = serde_json::from_str(&access_response)?;
            let access_token = access_json.access_token;

            let user_response = client
                .get("https://api.linkedin.com/v2/userinfo")
                .bearer_auth(access_token)
                .send()
                .await?
                .text()
                .await?;

            let account_data: LinkedInUserResponse = serde_json::from_str(&user_response)?;

            let linked_account = LinkedAccount {
                id: account_data.sub,
                name: LinkedService::LinkedIn,
                email: account_data.email.unwrap_or_default(),
                url: "".to_string(),
                avatar: account_data.picture.unwrap_or_default(),
                is_public: false,
                username: account_data.name,
                token: None,
                scope: None,
            };

            if Self::find_user_by_linked_account(
                &self,
                linked_account.id.clone(),
                &LinkedService::LinkedIn,
            ).await?.is_some() {
                return Err(anyhow::anyhow!("Account has already been added").code(404))
            }

            Self::add_linked_account(&self, id, linked_account.clone(), auth).await?;

            return Ok(PublicLinkedAccount::from(linked_account))
        }

        Err(anyhow::anyhow!("Error adding account").code(404))
    }

    pub async fn delete_linked_account(
        &self,
        user_id: ObjectId,
        account_id: String,
    ) -> error::Result<PublicLinkedAccount> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(mut user) = users.find("id", &Bson::ObjectId(user_id)).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        if !Edit.get_access(&auth, &user) {
            return Err(anyhow::anyhow!("User is not available to change this user").code(403));
        }

        if let Some(ref mut linked_accounts) = user.linked_accounts {
            let Some(account) = linked_accounts
                .iter()
                .find(|account| account.id == account_id)
                .cloned()
            else {
                return Err(anyhow::anyhow!("No linked account found").code(404));
            };

            linked_accounts.retain(|account| account.id != account_id);
            user.last_modified = Utc::now().timestamp_micros();

            users.delete("id", &user_id).await?;
            users.insert(&user).await?;

            return Ok(PublicLinkedAccount::from(account));
        }

        Err(anyhow::anyhow!("No linked account found").code(404))
    }

    pub async fn change_linked_account(
        &self,
        user_id: ObjectId,
        account_id: String,
        data: UpdateLinkedAccount,
    ) -> error::Result<PublicLinkedAccount> {
        let auth = self.context.auth();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(mut user) = users.find("id", &Bson::ObjectId(user_id)).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        if !Edit.get_access(&auth, &user) {
            return Err(anyhow::anyhow!("User is not available to change this user").code(403));
        }

        if let Some(ref mut linked_accounts) = user.linked_accounts {
            if let Some(mut account) = linked_accounts
                .iter()
                .find(|account| account.id == account_id)
                .cloned()
            {
                if let Some(is_public) = data.is_public {
                    account.is_public = is_public;
                }

                if let Some(update_token) = data.update_token {
                    if update_token {
                        account.token = data.token;
                        account.scope = data.scope;
                    }
                }

                if let Some(idx) = linked_accounts
                    .iter()
                    .position(|acc| acc.id == account_id)
                {
                    linked_accounts[idx] = account.clone();
                }

                user.last_modified = Utc::now().timestamp_micros();

                users.delete("id", &user_id).await?;
                users.insert(&user).await?;

                return Ok(PublicLinkedAccount::from(account));
            }
        }

        Err(anyhow::anyhow!("No linked account found").code(404))
    }

    pub async fn add_wallet(
        &self,
        user_id: ObjectId,
        data: AddWallet,
    ) -> error::Result<PublicLinkedAccount> {
        let auth = self.context.auth();

        let message = keccak256(
            format!(
                "{}{}{}",
                "\x19Ethereum Signed Message:\n",
                data.message.len(),
                data.message,
            ).as_bytes(),
        );
        let signature = match hex::decode(&data.signature[2..]) {
            Ok(sig) => sig,
            Err(e) => return Err(anyhow::anyhow!("Error decoding signature: {:?}", e).code(502)),
        };
        let recovery_id = match signature.get(64) {
            Some(byte) => byte.clone() as i32 - 27,
            None => return Err(anyhow::anyhow!("Invalid signature format").code(502)),
        };
        let pubkey = match recover(&message, &signature[..64], recovery_id) {
            Ok(pubkey) => pubkey,
            Err(e) => return Err(anyhow::anyhow!("Error recovering public key: {:?}", e).code(502)),
        };
        let pubkey = format!("{:02X?}", pubkey);

        if data.address.to_lowercase() == pubkey.to_lowercase() {
            let linked_account = LinkedAccount {
                id: data.address.clone(),
                name: LinkedService::WalletConnect,
                email: "".to_string(),
                url: "".to_string(),
                avatar: "".to_string(),
                is_public: false,
                username: data.address,
                token: None,
                scope: None,
            };

            if Self::find_user_by_linked_account(
                &self,
                linked_account.id.clone(),
                &LinkedService::WalletConnect,
            ).await?.is_some() {
                return Err(anyhow::anyhow!("Account has already been added").code(404))
            }

            Self::add_linked_account(&self, user_id, linked_account.clone(), auth).await?;

            return Ok(PublicLinkedAccount::from(linked_account))
        }

        Err(anyhow::anyhow!("Error adding account wallet").code(404))
    }

    pub async fn proxy_github_api(
        &self,
        path: String,
        query: Vec<(String, String)>
    ) -> error::Result<HttpResponse> {
        let auth = self.context.auth();
        let id = auth.id().unwrap();

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users.find("id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        let (access_token, github_account_id) = get_access_token(user);

        let request = create_request_with_token(
            format!("https://api.github.com/{}", path),
            query,
            access_token,
        ).await?;

        let github_response = request
            .send()
            .await?;

        let status_code = github_response.status();
        let mut response = HttpResponse::build(status_code);

        for (name, value) in github_response.headers() {
            response.append_header((name.clone(), value.clone()));
        }

        response.append_header(("Content-Type", "application/json"));

        let body = github_response.text().await?;
        let response = response.body(body.clone());

        if status_code == StatusCode::UNAUTHORIZED
            && body.contains("\"Bad credentials\"")
            && github_account_id.is_some() {
            let _ = self.context
                .make_request()
                .patch(format!(
                    "{}://{}/{}/user/{}/linked_account/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    id,
                    github_account_id.unwrap(),
                ))
                .auth(self.context.server_auth())
                .json(&UpdateLinkedAccount {
                    update_token: Some(true),
                    ..UpdateLinkedAccount::default()
                })
                .send()
                .await
                .unwrap();
        }

        Ok(response)
    }

    pub async fn proxy_github_files(
        &self,
        path: String,
        query: Vec<(String, String)>
    ) -> error::Result<HttpResponse> {
        log::info!("proxy_github_files RUN");

        if path.starts_with("raw.githubusercontent.com") {
            let auth = self.context.auth();
            let id = auth.id().unwrap();

            let users = self.context.try_get_repository::<User<ObjectId>>()?;

            let Some(user) = users.find("id", &Bson::ObjectId(id)).await? else {
                return Err(anyhow::anyhow!("No user found").code(404));
            };

            let (access_token, _) = get_access_token(user);

            let request = create_request_with_token(
                format!("https://{}", path),
                query,
                access_token,
            ).await?;

            let github_response = request
                .send()
                .await?;

            let status_code = github_response.status();
            let mut response = HttpResponse::build(status_code);

            let body = github_response.bytes().await?;
            let response = response.body(body.clone());

            return Ok(response)
        }

        Err(anyhow::anyhow!("Only for GitHub raw").code(404))
    }
}
