use anyhow::bail;
use chrono::Utc;
use common::{
    auth::Auth,
    context::Context,
    entities::{letter::CreateLetter, user::User},
    repository::Entity,
    services::{MAIL_SERVICE, PROTOCOL, USERS_SERVICE},
};
use mongodb::bson::{oid::ObjectId, Bson};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use super::user::{CreateUser, PublicUser};

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
    pub user: PublicUser,
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
            user: user.into(),
            token: auth.to_token()?,
        })
    }

    pub async fn authentication(
        &self,
        mut user: CreateUser,
        verify_email: bool,
    ) -> anyhow::Result<PublicUser> {
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

        Ok(link.user.into())
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
}
