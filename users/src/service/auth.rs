use anyhow::bail;
use common::{auth::Auth, context::Context, entities::user::User};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

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

        if !Self::request_access(login.password.clone(), user.password, user.salt) {
            bail!("Incorrect password")
        }
        let auth = Auth::User(user.id);

        Ok(Token {
            token: auth.to_token()?,
        })
    }
}
