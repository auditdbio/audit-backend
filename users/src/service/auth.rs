use chrono::Utc;
use common::{
    access_rules::AccessRules,
    auth::Auth,
    context::Context,
    entities::{letter::CreateLetter, user::User},
    error::{self, AddCode},
    repository::Entity,
    services::{FRONTEND, MAIL_SERVICE, PROTOCOL, USERS_SERVICE},
};
use mongodb::bson::{oid::ObjectId, Bson};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use super::user::CreateUser;

pub struct AuthService {
    context: Context,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub token: String,
    pub user: User<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub user: User<ObjectId>,
    pub code: String,
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

impl AuthService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    fn request_access(mut auth_password: String, correct_password: &String, salt: &str) -> bool {
        auth_password.push_str(salt);
        &sha256::digest(auth_password) == correct_password
    }

    pub async fn login(&self, login: &Login) -> error::Result<Token> {
        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        let Some(user) = users.find("email", &Bson::String(login.email.clone())).await? else {
            return Err(anyhow::anyhow!("No user found").code(404));
        };

        if !Self::request_access(login.password.clone(), &user.password, &user.salt) {
            return Err(anyhow::anyhow!("Incorrect password").code(401));
        }
        let auth = Auth::User(user.id);

        Ok(Token {
            user: user.stringify(),
            token: auth.to_token()?,
        })
    }

    pub async fn authentication(
        &self,
        mut user: CreateUser,
        verify_email: bool,
    ) -> error::Result<User<String>> {
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

        let user = User {
            id: ObjectId::new(),
            name: user.name,
            email: user.email,
            salt,
            password,
            current_role: user.current_role,
            last_modified: Utc::now().timestamp_micros(),
            is_new: true,
        };

        let link = Link {
            user,
            code: code.clone(),
        };

        if verify_email {
            links.insert(&link).await?;
        } else {
            users.insert(&link.user).await?;
        }

        Ok(link.user.stringify())
    }

    pub async fn verify_link(&self, code: String) -> error::Result<bool> {
        let codes = self.context.try_get_repository::<Link>()?;

        // TODO: rewrite this with get_access

        let Some(link) = codes.find("code", &Bson::String(code.clone())).await? else {
            return Ok(false);
        };

        let users = self.context.try_get_repository::<User<ObjectId>>()?;

        users.insert(&link.user).await?;

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

        let Some(code) = codes.find("code", &Bson::String(token.code.clone())).await? else {
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

        users.insert(&user).await?;
        Ok(())
    }
}
