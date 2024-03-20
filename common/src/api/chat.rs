use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    api::{
        audits::PublicAudit,
        requests::PublicRequest,
    },
    entities::{
        audit::AuditStatus,
        audit_request::TimeRange,
        contacts::Contacts,
        role::Role,
    },
    error::{self, AddCode},
    services::{API_PREFIX, CHAT_SERVICE, PROTOCOL},
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
pub struct PublicReadId {
    pub id: String,
    pub unread: i32,
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
pub struct PublicChat {
    pub id: String,
    pub name: String,
    pub members: Vec<PublicChatId>,
    pub last_modified: i64,
    pub last_message: PublicMessage,
    pub avatar: Option<String>,
    pub unread: Vec<PublicReadId>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AuditMessageStatus {
    Request,
    Waiting,
    Started,
    Resolved,
    Declined,
}

impl From<AuditStatus> for AuditMessageStatus {
    fn from(status: AuditStatus) -> Self {
        match status {
            AuditStatus::Waiting => AuditMessageStatus::Waiting,
            AuditStatus::Started => AuditMessageStatus::Started,
            AuditStatus::Resolved => AuditMessageStatus::Resolved,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMessage {
    pub id: String,
    pub customer_id: String,
    pub auditor_id: String,
    pub project_id: String,
    pub project_name: String,
    pub description: String,
    pub project_scope: Vec<String>,
    pub tags: Option<Vec<String>>,
    pub price: i64,
    pub status: Option<AuditMessageStatus>,
    pub auditor_first_name: String,
    pub auditor_last_name: String,
    pub avatar: String,
    pub auditor_contacts: Contacts,
    pub customer_contacts: Contacts,
    pub last_changer: Role,
    pub time: TimeRange,
    pub report: Option<String>,
    pub report_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditMessageId {
    pub chat_id: String,
    pub message_id: String,
}

pub fn send_message(message: CreateMessage, auth: Auth) -> error::Result<PublicChat> {
    let res = ureq::post(&format!(
        "{}://{}/{}/chat/message",
        PROTOCOL.as_str(),
        CHAT_SERVICE.as_str(),
        API_PREFIX.as_str(),
    ))
    .set("Authorization", &format!("Bearer {}", auth.to_token()?))
    .send_json(message)?;

    if res.status() >= 200 && res.status() < 300 {
        Ok(res.into_json::<PublicChat>()?)
    } else {
        Err(anyhow::anyhow!("Error sending message").code(400))
    }
}

pub fn delete_message(chat_id: String, message_id: String, auth: Auth) -> error::Result<()> {
    let _ = ureq::delete(&format!(
        "{}://{}/{}/chat/{}/message/{}",
        PROTOCOL.as_str(),
        CHAT_SERVICE.as_str(),
        API_PREFIX.as_str(),
        chat_id,
        message_id,
    ))
    .set("Authorization", &format!("Bearer {}", auth.to_token()?))
    .call()?;

    Ok(())
}

pub enum CreateAuditMessage {
    Request(PublicRequest),
    Audit(PublicAudit),
}

pub fn create_audit_message(
    audit: CreateAuditMessage,
    status: Option<AuditMessageStatus>,
    receiver_id: ObjectId,
    receiver_role: Role,
    last_changer: Role,
) -> CreateMessage {
    match audit {
        CreateAuditMessage::Request(request) => {
            let message_text = AuditMessage {
                id: request.id.clone(),
                customer_id: request.customer_id.clone(),
                auditor_id: request.auditor_id.clone(),
                project_id: request.project_id.clone(),
                project_name: request.project_name.clone(),
                description: request.description,
                project_scope: request.project_scope,
                tags: request.tags,
                price: request.price.clone(),
                status,
                auditor_first_name: request.auditor_first_name,
                auditor_last_name: request.auditor_last_name,
                avatar: request.avatar,
                auditor_contacts: request.auditor_contacts,
                customer_contacts: request.customer_contacts,
                last_changer,
                time: request.time.clone(),
                report: None,
                report_name: None,
            };

            let message = CreateMessage {
                chat: None,
                to: Some(PublicChatId {
                    role: receiver_role,
                    id: receiver_id.to_hex(),
                }),
                role: last_changer,
                text: serde_json::to_string(&message_text).unwrap(),
                kind: Some(MessageKind::Audit),
            };
            message
        }
        CreateAuditMessage::Audit(audit) => {
            let message_text = AuditMessage {
                id: audit.id.clone(),
                customer_id: audit.customer_id.clone(),
                auditor_id: audit.auditor_id.clone(),
                project_id: audit.project_id.clone(),
                project_name: audit.project_name.clone(),
                description: audit.description,
                project_scope: audit.scope,
                tags: Some(audit.tags),
                price: audit.price.clone(),
                status,
                auditor_first_name: audit.auditor_first_name,
                auditor_last_name: audit.auditor_last_name,
                avatar: audit.avatar,
                auditor_contacts: audit.auditor_contacts,
                customer_contacts: audit.customer_contacts,
                last_changer,
                time: audit.time.clone(),
                report: audit.report,
                report_name: audit.report_name,
            };

            let message = CreateMessage {
                chat: None,
                to: Some(PublicChatId {
                    role: receiver_role,
                    id: receiver_id.to_hex(),
                }),
                role: last_changer,
                text: serde_json::to_string(&message_text).unwrap(),
                kind: Some(MessageKind::Audit),
            };
            message
        }
    }
}
