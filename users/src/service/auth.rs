use actix_web::HttpRequest;
use chrono::Utc;
use common::{
    access_rules::AccessRules,
    api::{
        self,
        badge::{BadgePayload, get_badge},
        codes::post_code,
        user::CreateUser,
        linked_accounts::{
            LinkedService, GithubUserEmails,
            GetGithubAccessToken, GithubAccessResponse,
            GithubUserData, AddLinkedAccount,
            UpdateLinkedAccount,
        },
    },
    auth::Auth,
    context::GeneralContext,
    entities::{
        letter::CreateLetter,
        user::{LinkedAccount, User, UserLogin},
    },
    error::{self, AddCode},
    repository::Entity,
    services::{FRONTEND, MAIL_SERVICE, PROTOCOL, USERS_SERVICE},
};
use mongodb::bson::{Bson, oid::ObjectId};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::env::var;

extern crate crypto;
use crypto::buffer::{ RefReadBuffer, RefWriteBuffer, BufferResult };

use super::user::UserService;

lazy_static::lazy_static! {
    static ref ADMIN_CREATION_PASSWORD: String = std::env::var("ADMIN_CREATION_PASSWORD").unwrap();
}

pub struct AuthService {
    context: GeneralContext,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub token: String,
    pub user: UserLogin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub user: User<ObjectId>,
    pub code: String,
    pub secret: Option<String>,
}

impl Entity for Link {
    fn id(&self) -> ObjectId {
        ObjectId::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Code {
    pub code: String,
    pub user: ObjectId,
}

impl Entity for Code {
    fn id(&self) -> ObjectId {
        self.user
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordData {
    pub code: String,
    pub password: String,
}

pub struct ChangePassword;

impl<'b> AccessRules<String, &'b User<ObjectId>> for ChangePassword {
    fn get_access(&self, new_password: String, user: &'b User<ObjectId>) -> bool {
        AuthService::request_access(new_password, &user.password, &user.salt)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponce {
    pub token: String,
}

impl AuthService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    fn request_access(mut auth_password: String, correct_password: &String, salt: &str) -> bool {
        auth_password.push_str(salt);
        &sha256::digest(auth_password) == correct_password
    }

    pub async fn login(&self, login: &Login) -> error::Result<Token> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users
            .find("email", &Bson::String(login.email.clone()))
            .await?
        else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        if let Some(is_passwordless) = user.is_passwordless {
            if is_passwordless {
                return Err(anyhow::anyhow!("Incorrect password").code(401));
            }
        }

        if !Self::request_access(login.password.clone(), &user.password, &user.salt) {
            return Err(anyhow::anyhow!("Incorrect password").code(401));
        }

        let auth = if user.is_admin {
            Auth::Admin(user.id)
        } else {
            Auth::User(user.id)
        };

        Ok(Token {
            user: UserLogin::from(user),
            token: auth.to_token()?,
        })
    }

    pub async fn authentication(
        &self,
        mut user: CreateUser,
        mut verify_email: bool,
    ) -> error::Result<User<String>> {
        if let Some(secret) = &user.secret {
            log::info!("Secret: {}", secret);
            let payload: BadgePayload = serde_json::from_str(
                &api::codes::get_code(&self.context, secret.clone())
                    .await?
                    .unwrap(),
            )?;

            verify_email &= payload.email != user.email;
        }

        let is_admin: bool =
            user.admin_creation_password == Some(ADMIN_CREATION_PASSWORD.to_string());

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let links = self.context.try_get_repository::<Link>()?;

        if users
            .find("email", &Bson::String(user.email.clone()))
            .await?
            .is_some()
            || links
                .find("user.email", &Bson::String(user.email.clone()))
                .await?
                .is_some()
        {
            return Err(anyhow::anyhow!("User with email already exists").code(409));
        }

        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        if verify_email {
            let message = include_str!("../../templates/link.txt")
                .to_string()
                .replace(
                    "{link}",
                    format!(
                        "{}://{}/api/auth/verify/{}",
                        PROTOCOL.as_str(),
                        USERS_SERVICE.as_str(),
                        code
                    )
                    .as_str(),
                );

            let letter = CreateLetter {
                email: user.email.clone(),
                message,
                subject: "Registration at auditdb.io".to_string(),
                recipient_id: None,
                recipient_name: Some(user.name.clone()),
            };

            self.context
                .make_request()
                .auth(self.context.server_auth())
                .post(format!(
                    "{}://{}/api/mail",
                    PROTOCOL.as_str(),
                    MAIL_SERVICE.as_str()
                ))
                .json(&letter)
                .send()
                .await?;
        }

        let salt: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        user.password.push_str(&salt);
        let password = sha256::digest(user.password);

        let new_user = User {
            id: ObjectId::new(),
            name: user.name,
            email: user.email,
            salt,
            password,
            current_role: user.current_role,
            last_modified: Utc::now().timestamp_micros(),
            created_at: Some(Utc::now().timestamp_micros()),
            is_new: true,
            is_admin,
            linked_accounts: user.linked_accounts,
            is_passwordless: user.is_passwordless,
        };

        let link = Link {
            user: new_user,
            code: code.clone(),
            secret: user.secret,
        };

        if verify_email {
            links.insert(&link).await?;
        } else {
            UserService::new(self.context.clone())
                .create(link.user.clone(), link.secret)
                .await?;
        }

        Ok(link.user.stringify())
    }

    pub async fn github_get_user(
        &self,
        data: GetGithubAccessToken,
        current_role: String,
    ) -> error::Result<(CreateUser, LinkedAccount)> {
        let client = Client::new();

        let access_response = client
            .post(format!(
                "https://github.com/login/oauth/access_token?code={}&client_id={}&client_secret={}",
                data.code, data.client_id, data.client_secret,
            ))
            .header(header::ACCEPT, "application/json")
            .send()
            .await?
            .text()
            .await?;

        let access_json: GithubAccessResponse = serde_json::from_str(&access_response)?;
        let access_token = access_json.access_token;

        let user_response = client
            .get("https://api.github.com/user")
            .header(header::ACCEPT, "application/json")
            .header("User-Agent", "auditdbio")
            .bearer_auth(access_token.clone())
            .send()
            .await?
            .text()
            .await?;

        let emails_response = client
            .get("https://api.github.com/user/emails")
            .header(header::ACCEPT, "application/json")
            .header("User-Agent", "auditdbio")
            .bearer_auth(access_token.clone())
            .send()
            .await?
            .text()
            .await?;

        let user_data: GithubUserData = serde_json::from_str(&user_response)?;
        let emails: Vec<GithubUserEmails> = serde_json::from_str(&emails_response)?;

        let Some(email) = emails
            .iter()
            .find(|email| email.primary)
            .map(|email| email.email.to_string())
        else {
            return Err(anyhow::anyhow!("No email found").code(404));
        };

        let key = var("GITHUB_TOKEN_CRYPTO_KEY").unwrap();
        let iv = var("GITHUB_TOKEN_CRYPTO_IV").unwrap();

        let mut encryptor = crypto::aes::cbc_encryptor(
            crypto::aes::KeySize::KeySize256,
            key.as_bytes(),
            iv.as_bytes(),
            crypto::blockmodes::PkcsPadding,
        );

        let mut buffer = Vec::with_capacity(2 * access_token.len());
        let mut read_buffer = RefReadBuffer::new(access_token.as_bytes());

        if buffer.capacity() < access_token.len() {
            return Err(anyhow::anyhow!("Buffer size is insufficient for encryption").code(500));
        }

        let ciphertext = {
            let mut write_buffer = RefWriteBuffer::new(&mut buffer);
            encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)
        };

        let token = if let Ok(BufferResult::BufferUnderflow) = ciphertext {
            Some(buffer)
        } else {
            None
        };

        let linked_account = LinkedAccount {
            id: user_data.id.to_string(),
            name: LinkedService::GitHub,
            email: email.clone(),
            url: user_data.html_url,
            avatar: user_data.avatar_url,
            is_public: false,
            username: user_data.login.clone(),
            token,
        };

        let user = CreateUser {
            email,
            password: "".to_string(),
            name: user_data.login,
            current_role,
            use_email: None,
            admin_creation_password: None,
            secret: None,
            linked_accounts: Some(vec![linked_account.clone()]),
            is_passwordless: Some(true),
        };

        return Ok((user, linked_account));
    }

    pub async fn github_auth(&self, data: AddLinkedAccount) -> error::Result<Token> {
        let github_auth = GetGithubAccessToken {
            code: data.clone().code,
            client_id: var("GITHUB_CLIENT_ID").unwrap(),
            client_secret: var("GITHUB_CLIENT_SECRET").unwrap(),
        };

        let user_service = UserService::new(self.context.clone());

        let (github_user, linked_account) =
            self.github_get_user(github_auth, data.clone().current_role).await?;

        if let Some(mut user) = user_service
            .find_user_by_linked_account(linked_account.id.clone(), &linked_account.name)
            .await?
        {
            user.current_role = data.clone().current_role;
            let _ = self.context
                .make_request()
                .patch(format!(
                    "{}://{}/api/user/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    user.id
                ))
                .auth(self.context.server_auth())
                .json(&data)
                .send()
                .await
                .unwrap();

            let _ = self.context
                .make_request()
                .patch(format!(
                    "{}://{}/api/user/{}/linked_account/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    user.id,
                    linked_account.id.clone(),
                ))
                .auth(self.context.server_auth())
                .json(&UpdateLinkedAccount {
                    is_public: None,
                    token: linked_account.token
                })
                .send()
                .await
                .unwrap();

            return create_auth_token(&user);
        }

        let existing_email_user = user_service
            .find_by_email(github_user.email.clone())
            .await?;

        if let Some(mut user) = existing_email_user {
            let _ = user_service
                .add_linked_account(user.id, linked_account, self.context.server_auth())
                .await?;

            user.current_role = data.clone().current_role;
            let _ = self.context
                .make_request()
                .patch(format!(
                    "{}://{}/api/user/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    user.id
                ))
                .auth(self.context.server_auth())
                .json(&data)
                .send()
                .await
                .unwrap();

            return create_auth_token(&user);
        }

        let verify_email = false;
        self.authentication(github_user.clone(), verify_email)
            .await?;

        if let Some(user) = user_service
            .find_by_email(github_user.email.clone())
            .await?
        {
            return create_auth_token(&user);
        }

        Err(anyhow::anyhow!("Login failed").code(404))
    }

    pub async fn verify_link(&self, code: String) -> error::Result<bool> {
        let codes = self.context.try_get_repository::<Link>()?;

        // TODO: rewrite this with get_access

        let Some(link) = codes.find("code", &Bson::String(code.clone())).await? else {
            return Ok(false);
        };

        let user = link.user;

        let merge_secret = if let Some(secret) = &link.secret {
            Some(secret.clone())
        } else {
            // get badge by email
            let badge = get_badge(&self.context, user.email.clone()).await?;
            if let Some(badge) = badge {
                let payload = BadgePayload {
                    badge_id: badge.user_id.parse()?,
                    email: badge.contacts.email.unwrap(),
                };

                Some(post_code(&self.context, serde_json::to_string(&payload)?).await?)
            } else {
                None
            }
        };

        UserService::new(self.context.clone())
            .create(user, merge_secret)
            .await?;

        Ok(true)
    }

    pub async fn forgot_password(&self, email: String) -> error::Result<()> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users.find("email", &Bson::String(email.clone())).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let message = include_str!("../../templates/password_change.txt")
            .to_string()
            .replace(
                "{link}",
                format!(
                    "{}://{}/restore-password/{}",
                    PROTOCOL.as_str(),
                    FRONTEND.as_str(),
                    code
                )
                .as_str(),
            );

        let code = Code {
            code: code.clone(),
            user: user.id,
        };

        let codes = self.context.try_get_repository::<Code>()?;

        codes.insert(&code).await?;

        let letter = CreateLetter {
            email: user.email.clone(),
            message,
            subject: "Password change at auditdb.io".to_string(),
            recipient_id: Some(user.id),
            recipient_name: Some(user.name.clone()),
        };

        self.context
            .make_request()
            .auth(self.context.server_auth())
            .post(format!(
                "{}://{}/api/mail",
                PROTOCOL.as_str(),
                MAIL_SERVICE.as_str()
            ))
            .json(&letter)
            .send()
            .await?;

        Ok(())
    }

    pub async fn reset_password(&self, token: ChangePasswordData) -> error::Result<()> {
        let codes = self.context.try_get_repository::<Code>()?;

        let Some(code) = codes
            .find("code", &Bson::String(token.code.clone()))
            .await?
        else {
            return Err(anyhow::anyhow!("No code found").code(404));
        };

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(mut user) = users.delete("id", &code.user).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        let salt: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let new_password = sha256::digest(format!("{}{}", token.password, salt));

        user.password = new_password;
        user.salt = salt;
        user.is_passwordless = Some(false);

        users.insert(&user).await?;
        Ok(())
    }

    pub async fn restore(&self, req: HttpRequest) -> error::Result<TokenResponce> {
        let Some(token) = req.headers().get("Authorization") else {
            return Err(anyhow::anyhow!("No token found").code(401));
        };
        let token = token.to_str()?.strip_prefix("Bearer ").unwrap();
        let auth = Auth::from_token(token)?;
        if auth.is_some() && auth != Some(Auth::None) {
            return Ok(TokenResponce {
                token: auth.unwrap().to_token()?,
            });
        }
        Err(anyhow::anyhow!("Invalid token").code(401))
    }
}

pub fn create_auth_token(user: &User<ObjectId>) -> error::Result<Token> {
    let auth = if user.is_admin {
        Auth::Admin(user.id)
    } else {
        Auth::User(user.id)
    };
    Ok(Token {
        user: UserLogin::from(user.clone()),
        token: auth.to_token()?,
    })
}
