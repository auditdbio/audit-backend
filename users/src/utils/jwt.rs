use crate::error::Result;
use crate::prelude::*;

use jsonwebtoken::{encode, Header, EncodingKey, decode, Validation, TokenData, DecodingKey};
use serde::{Serialize, Deserialize};

use crate::repositories::user::UserModel;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub user_id: String,
    pub token: String,
}

static SECRET: &'static str = "I will do this later";

pub fn create(claims: Claims) -> Result<String> {
    let key = EncodingKey::from_secret(SECRET.as_bytes());

    let jwt = encode(&Header::default(), &claims, &key).is_signature()?.unwrap();
    Ok(jwt)
}

pub fn verify(jwt: &str) -> Result<Option<Claims>> {

    let key = DecodingKey::from_secret(SECRET.as_bytes());

    let jwt: Option<TokenData<Claims>> = decode::<Claims>(jwt, &key, &Validation::default()).is_signature()?;

    Ok(jwt.map(|jwt| jwt.claims))
}
