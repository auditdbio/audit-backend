use anyhow::bail;
use chrono::Utc;
use common::{
    auth::Auth,
    context::Context,
    entities::{letter::CreateLetter, user::User},
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
        self.user.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePassword {
    pub code: String,
    pub password: String,
}

impl AuthService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    fn request_access(mut auth_password: String, correct_password: String, salt: String) -> bool {
        auth_password.push_str(&salt);
        sha256::digest(auth_password) == correct_password
    }

    pub async fn login(&self, login: &Login) -> anyhow::Result<Token> {
        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(user) = users.find("email", &Bson::String(login.email.clone())).await? else {
            bail!("No user found")
        };

        if !Self::request_access(
            login.password.clone(),
            user.password.clone(),
            user.salt.clone(),
        ) {
            bail!("Incorrect password")
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
    ) -> anyhow::Result<User<String>> {
        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(links) = self.context.get_repository::<Link>() else {
            bail!("No code repository found")
        };

        if users
            .find("email", &Bson::String(user.email.clone()))
            .await?
            .is_some()
            || links
                .find("user.email", &Bson::String(user.email.clone()))
                .await?
                .is_some()
        {
            bail!("User with email already exists")
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

    pub async fn verify_link(&self, code: String) -> anyhow::Result<bool> {
        let Some(codes) = self.context.get_repository::<Link>() else {
            bail!("No code repository found")
        };

        // TODO: rewrite this with get_access

        let Some(link) = codes.find("code", &Bson::String(code.clone())).await? else {
            return Ok(false);
        };

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        users.insert(&link.user).await?;

        Ok(true)
    }

    pub async fn forgot_password(&self, email: String) -> anyhow::Result<()> {
        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(user) = users.find("email", &Bson::String(email.clone())).await? else {
            bail!("No user found")
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
                    "{}://{}/change-password/{}",
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

        let Some(codes) = self.context.get_repository::<Code>() else {
            bail!("No code repository found")
        };

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

    pub async fn reset_password(&self, token: ChangePassword) -> anyhow::Result<()> {
        let Some(codes) = self.context.get_repository::<Code>() else {
            bail!("No code repository found")
        };

        let Some(code) = codes.find("code", &Bson::String(token.code.clone())).await? else {
            bail!("No code found")
        };

        let Some(users) = self.context.get_repository::<User<ObjectId>>() else {
            bail!("No user repository found")
        };

        let Some(mut user) = users.delete("id", &code.user).await? else {
            bail!("No user found")
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
