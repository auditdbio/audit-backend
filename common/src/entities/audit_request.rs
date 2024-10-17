use std::collections::HashMap;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    impl_has_last_modified,
    api::chat::AuditMessageId,
    entities::audit::AuditEditHistory,
    repository::{Entity, HasLastModified},
};

use super::role::Role;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone, Default)]
pub struct PriceRange {
    pub from: i64,
    pub to: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
pub struct TimeRange {
    pub from: i64,
    pub to: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub struct AuditRequest<Id> {
    pub id: Id,
    pub auditor_id: Id,
    pub customer_id: Id,
    pub project_id: Id,

    pub description: String,
    pub tags: Option<Vec<String>>,
    pub scope: Option<Vec<String>>,

    pub price: Option<i64>,
    pub total_cost: Option<i64>,
    pub last_modified: i64,
    pub last_changer: Role,
    pub time: TimeRange,

    pub chat_id: Option<AuditMessageId>,

    #[serde(default)]
    pub edit_history: Vec<AuditEditHistory>,
    #[serde(default)]
    pub unread_edits: HashMap<String, usize>,

    pub auditor_organization: Option<Id>,
    pub customer_organization: Option<Id>,
}

impl_has_last_modified!(AuditRequest<ObjectId>);

impl AuditRequest<String> {
    pub fn parse(self) -> AuditRequest<ObjectId> {
        AuditRequest {
            id: self.id.parse().unwrap(),
            auditor_id: self.auditor_id.parse().unwrap(),
            customer_id: self.customer_id.parse().unwrap(),
            project_id: self.project_id.parse().unwrap(),
            description: self.description,
            tags: self.tags,
            scope: self.scope,
            price: self.price,
            total_cost: self.total_cost,
            last_modified: self.last_modified,
            last_changer: self.last_changer,
            time: self.time,
            chat_id: self.chat_id,
            edit_history: self.edit_history,
            unread_edits: self.unread_edits,
            auditor_organization: self.auditor_organization.map(|v| v.parse().unwrap()),
            customer_organization: self.customer_organization.map(|v| v.parse().unwrap()),
        }
    }
}

impl AuditRequest<ObjectId> {
    pub fn stringify(self) -> AuditRequest<String> {
        AuditRequest {
            id: self.id.to_hex(),
            auditor_id: self.auditor_id.to_hex(),
            customer_id: self.customer_id.to_hex(),
            project_id: self.project_id.to_hex(),
            description: self.description,
            tags: self.tags,
            scope: self.scope,
            price: self.price,
            total_cost: self.total_cost,
            last_modified: self.last_modified,
            last_changer: self.last_changer,
            time: self.time,
            chat_id: self.chat_id,
            edit_history: self.edit_history,
            unread_edits: self.unread_edits,
            auditor_organization: self.auditor_organization.map(|v| v.to_hex()),
            customer_organization: self.customer_organization.map(|v| v.to_hex()),
        }
    }
}

impl Entity for AuditRequest<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}
