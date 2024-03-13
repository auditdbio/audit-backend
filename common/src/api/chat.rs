use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    entities::{
        audit::AuditStatus,
        role::Role,
    },
    error::{self, AddCode},
    services::{API_PREFIX, CHAT_SERVICE, PROTOCOL},
};
use crate::entities::audit_request::TimeRange;

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
pub enum MessageKind {
    Text,
    Image,
    File,
    Audit,
    AuditIssue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub chat: Option<String>,
    pub to: Option<PublicChatId>,
    pub role: Role,
    pub text: String,
    pub kind: Option<MessageKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicChatId {
    pub role: Role,
    pub id: String,
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
    pub kind: Option<MessageKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMessage {
    pub id: String,
    pub customer_id: String,
    pub auditor_id: String,
    pub project_name: String,
    pub price: i64,
    pub status: Option<AuditStatus>,
    pub last_changer: Role,
    pub time: TimeRange,
    pub report: Option<String>,
    pub report_name: Option<String>,
}

pub fn create_message(message: CreateMessage, auth: Auth) -> error::Result<String> {
    let res = ureq::post(&format!(
        "{}://{}/{}/chat/message",
        PROTOCOL.as_str(),
        CHAT_SERVICE.as_str(),
        API_PREFIX.as_str(),
    ))
    .set("Authorization", &format!("Bearer {}", auth.to_token()?))
    .send_json(message)?;

    if res.status() >= 200 && res.status() < 300 {
        Ok(res.into_string()?)
    } else {
        Err(anyhow::anyhow!("Error sending message").code(400))
    }
}
