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

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PublicEvent {
    pub user_id: ObjectId,
    pub payload: EventPayload,
}
