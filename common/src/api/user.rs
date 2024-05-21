use mongodb::bson::oid::ObjectId;
use regex::Regex;
use serde::{Deserialize, Serialize};
use rand::{distributions::Alphanumeric, Rng};
use std::env::var;
use crypto::{ aes, blockmodes, buffer::{self, ReadBuffer, WriteBuffer, BufferResult}};

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::{
        auditor::ExtendedAuditor,
        customer::PublicCustomer,
        user::{LinkedAccount, PublicUser, User}
    },
    error::{self, AddCode},
    services::{API_PREFIX, PROTOCOL, USERS_SERVICE, AUDITORS_SERVICE, CUSTOMERS_SERVICE},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub name: String,
    pub current_role: String,
    pub use_email: Option<bool>,
    pub admin_creation_password: Option<String>,
    pub secret: Option<String>,
    pub linked_accounts: Option<Vec<LinkedAccount>>,
    pub is_passwordless: Option<bool>,
}

pub async fn get_by_id(
    context: &GeneralContext,
    auth: Auth,
    id: ObjectId,
) -> error::Result<PublicUser> {
    Ok(context
        .make_request::<PublicUser>()
        .get(format!(
            "{}://{}/{}/user/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            id
        ))
        .auth(auth)
        .send()
        .await?
        .json::<PublicUser>()
        .await?)
}

pub async fn get_by_email(
    context: &GeneralContext,
    email: String,
) -> error::Result<Option<User<ObjectId>>> {
    Ok(context
        .make_request::<User<ObjectId>>()
        .get(format!(
            "{}://{}/{}/user_by_email/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            email
        ))
        .auth(context.server_auth())
        .send()
        .await?
        .json::<User<ObjectId>>()
        .await
        .ok())
}

pub async fn new_link_id(
    context: &GeneralContext,
    link_id: String,
    user_id: ObjectId,
    add_postfix: bool,
) -> error::Result<String> {
    let auditor = context
        .make_request::<ExtendedAuditor>()
        .get(format!(
            "{}://{}/{}/auditor_by_link_id/{}",
            PROTOCOL.as_str(),
            AUDITORS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            link_id,
        ))
        .auth(context.server_auth())
        .send()
        .await?;

    let customer = context
        .make_request::<PublicCustomer>()
        .get(format!(
            "{}://{}/{}/customer_by_link_id/{}",
            PROTOCOL.as_str(),
            CUSTOMERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            link_id,
        ))
        .auth(context.server_auth())
        .send()
        .await?;

    let is_taken = if auditor.status().is_success() {
        auditor.json::<ExtendedAuditor>().await.map_or_else(
            |_| false,
            |auditor| auditor.user_id().clone() != user_id.to_hex()
        )
    } else if customer.status().is_success() {
        customer.json::<PublicCustomer>().await.map_or_else(
            |_| false,
            |customer| customer.user_id.clone() != user_id.to_hex()
        )
    } else { false };

    if !add_postfix && is_taken.clone() {
        return Err(anyhow::anyhow!("This link id is already taken").code(400));
    }

    if add_postfix && is_taken {
        let rnd: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        let result_link_id = format!(
            "{}-{}{}",
            link_id,
            user_id.to_hex().chars().rev().take(3).collect::<String>(),
            rnd,
        );
        return Ok(result_link_id.to_lowercase());
    }

    Ok(link_id.to_lowercase())
}

pub fn validate_name(name: &str) -> bool {
    let regex = Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
    regex.is_match(name)
}

pub async fn decrypt_github_token(encrypted_token: Vec<u8>) -> error::Result<String> {
    let key = var("GITHUB_TOKEN_CRYPTO_KEY").unwrap();
    let iv = var("GITHUB_TOKEN_CRYPTO_IV").unwrap();

    let mut decryptor = aes::cbc_decryptor(
        aes::KeySize::KeySize256,
        key.as_bytes(),
        iv.as_bytes(),
        blockmodes::PkcsPadding,
    );

    let mut decrypted_data = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(&encrypted_token);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = match decryptor.decrypt(
            &mut read_buffer,
            &mut write_buffer,
            true
        ) {
            Ok(value) => value,
            _ => return Err(anyhow::anyhow!("Decryption error").code(500))
        };

        decrypted_data.extend(write_buffer
            .take_read_buffer()
            .take_remaining()
            .iter()
            .map(|&i| i)
        );

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(String::from_utf8_lossy(&decrypted_data).to_string())
}
