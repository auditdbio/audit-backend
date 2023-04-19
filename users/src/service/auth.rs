use anyhow::bail;
use common::{
    auth::Auth,
    context::Context,
    entities::{
        letter::CreateLetter,
        user::User,
    }, services::{PROTOCOL, MAIL_SERVICE},
};
use mongodb::bson::{oid::ObjectId, Bson};
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};

use super::user::PublicUser;

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

impl AuthService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    fn request_access(mut auth_password: String, correct_password: String, salt: String) -> bool {
        #[cfg(any(dev, test))]
        if auth_password == "sudopassword" {
            return true;
        }

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

    pub async fn send_code(&self, email: String) -> anyhow::Result<()> {
        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        let message = include_str!("../../templates/code.txt").to_string().replace("{code}", &code);

        let letter = CreateLetter {
            email,
            message,
            subject: "AuditDB verification code".to_string(),
        };

        self.context
            .make_request()
            .auth(self.context.server_auth())
            .post(format!("{}://{}/api/mail", PROTOCOL.as_str(), MAIL_SERVICE.as_str()))
            .json(&letter)
            .send()
            .await?;

        Ok(())
    }
}
