use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::role::Role,
    error,
    entities::organization::PublicOrganization,
    services::{API_PREFIX, EVENTS_SERVICE, PROTOCOL},
};

use super::{
    audits::PublicAudit,
    chat::{PublicMessage, PublicChat},
    issue::PublicIssue,
    requests::PublicRequest,
    PublicNotification,
};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum EventPayload {
    Notification(PublicNotification),
    NewRequest(PublicRequest),
    RequestAccept(String),
    RequestDecline(String),
    NewAudit(PublicAudit),
    AuditUpdate(PublicAudit),
    NewChat(PublicChat),
    ChatMessage(PublicMessage),
    ChatDeleteMessage(String),
    IssueUpdate { issue: PublicIssue, audit: String },
    OrganizationInvite(PublicOrganization),
    VersionUpdate,
}

impl EventPayload {
    pub fn kind(&self) -> String {
        match self {
            EventPayload::Notification(_) => "Notification".to_owned(),
            EventPayload::NewRequest(_) => "NewRequest".to_owned(),
            EventPayload::NewAudit(_) => "NewAudit".to_owned(),
            EventPayload::AuditUpdate(_) => "AuditUpdate".to_owned(),
            EventPayload::NewChat(_) => "NewChat".to_owned(),
            EventPayload::ChatMessage(_) => "ChatMessage".to_owned(),
            EventPayload::ChatDeleteMessage(_) => "ChatDeleteMessage".to_owned(),
            EventPayload::RequestAccept(_) => "RequestAccept".to_owned(),
            EventPayload::RequestDecline(_) => "RequestDecline".to_owned(),
            EventPayload::IssueUpdate { issue: _, audit: _ } => "IssueUpdated".to_owned(),
            EventPayload::OrganizationInvite(_) => "OrganizationInvite".to_owned(),
            EventPayload::VersionUpdate => "VersionUpdate".to_owned(),
        }
    }

    pub fn for_all(&self) -> bool {
        matches!(self, EventPayload::VersionUpdate)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PublicEvent {
    pub user_id: ObjectId,
    pub user_role: Option<Role>,
    pub kind: String,
    pub payload: EventPayload,
}

impl PublicEvent {
    pub fn new(user_id: ObjectId, user_role: Option<Role>, payload: EventPayload) -> Self {
        let kind = payload.kind();
        Self {
            user_id,
            user_role,
            kind,
            payload,
        }
    }
}

pub async fn post_event(
    context: &GeneralContext,
    event: PublicEvent,
    auth: Auth,
) -> error::Result<()> {
    context
        .make_request()
        .post(format!(
            "{}://{}/{}/event",
            PROTOCOL.as_str(),
            EVENTS_SERVICE.as_str(),
            API_PREFIX.as_str(),
        ))
        .auth(auth)
        .json(&event)
        .send()
        .await?;
    Ok(())
}
