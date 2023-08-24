use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    entities::role::Role,
    error,
    services::{CHAT_SERVICE, PROTOCOL},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChatId {
    pub role: Role,
    pub id: ObjectId,
}

impl ChatId {
    pub fn publish(self) -> PublicChatId {
        PublicChatId {
            role: self.role,
            id: self.id.to_hex(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub chat: Option<String>,
    pub to: Option<PublicChatId>,
    pub role: Role,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicChatId {
    role: Role,
    id: String,
}

impl PublicChatId {
    pub fn parse(self) -> error::Result<ChatId> {
        Ok(ChatId {
            role: self.role,
            id: self.id.parse()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMessage {
    pub id: String,
    pub from: PublicChatId,
    pub chat: String,
    pub time: i64,
    pub text: String,
}

pub fn create_message(message: CreateMessage, auth: Auth) -> error::Result<()> {
    ureq::post(&format!(
        "{}://{}/api/chat/message",
        PROTOCOL.as_str(),
        CHAT_SERVICE.as_str()
    ))
    .set("Authorization", &format!("Bearer {}", auth.to_token()?))
    .send_json(message)?;
    Ok(())
}
