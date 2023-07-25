use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use super::{audits::PublicAudit, requests::PublicRequest, PublicNotification};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum EventPayload {
    Notification(PublicNotification),
    NewRequest(PublicRequest),
    NewAudit(PublicAudit),
    AuditUpdate(PublicAudit),
}

impl EventPayload {
    pub fn kind(&self) -> String {
        match self {
            EventPayload::Notification(_) => "Notification".to_owned(),
            EventPayload::NewRequest(_) => "NewRequest".to_owned(),
            EventPayload::NewAudit(_) => "NewAudit".to_owned(),
            EventPayload::AuditUpdate(_) => "AuditUpdate".to_owned(),
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
