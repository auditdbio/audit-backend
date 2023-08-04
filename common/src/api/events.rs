use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use super::{
    audits::PublicAudit, chat::PublicMessage, requests::PublicRequest, PublicNotification,
};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum EventPayload {
    Notification(PublicNotification),
    NewRequest(PublicRequest),
    RequestAccept(String),
    RequestDecline(String),
    NewAudit(PublicAudit),
    AuditUpdate(PublicAudit),
    ChatMessage(PublicMessage),
}

impl EventPayload {
    pub fn kind(&self) -> String {
        match self {
            EventPayload::Notification(_) => "Notification".to_owned(),
            EventPayload::NewRequest(_) => "NewRequest".to_owned(),
            EventPayload::NewAudit(_) => "NewAudit".to_owned(),
            EventPayload::AuditUpdate(_) => "AuditUpdate".to_owned(),
            EventPayload::ChatMessage(_) => "ChatMessage".to_owned(),
            EventPayload::RequestAccept(_) => "RequestAccept".to_owned(),
            EventPayload::RequestDecline(_) => "RequestDecline".to_owned(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PublicEvent {
    pub user_id: ObjectId,
    pub kind: String,
    pub payload: EventPayload,
}

impl PublicEvent {
    pub fn new(user_id: ObjectId, payload: EventPayload) -> Self {
        let kind = payload.kind();
        Self {
            user_id,
            kind,
            payload,
        }
    }
}
