use crate::error::Result;
use crate::prelude::*;

use common::auth_session::AuthSession;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

static SECRET: &'static str = "TODO: I will do this later";

pub fn create(session: AuthSession) -> Result<String> {
    let key = EncodingKey::from_secret(SECRET.as_bytes());

    let jwt = encode(&Header::default(), &session, &key)
        .is_signature()?
        .unwrap();
    Ok(jwt)
}

pub fn verify(jwt: &str) -> Result<Option<AuthSession>> {
    let key = DecodingKey::from_secret(SECRET.as_bytes());

    let jwt: Option<TokenData<AuthSession>> =
        decode::<AuthSession>(jwt, &key, &Validation::default()).is_signature()?;

    Ok(jwt.map(|jwt| jwt.claims))
}
