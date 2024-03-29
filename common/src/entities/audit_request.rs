use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::Entity;

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

    pub price: i64,
    pub last_modified: i64,
    pub last_changer: Role,
    pub time: TimeRange,
}

impl AuditRequest<String> {
    pub fn parse(self) -> AuditRequest<ObjectId> {
        AuditRequest {
            id: self.id.parse().unwrap(),
            auditor_id: self.auditor_id.parse().unwrap(),
            customer_id: self.customer_id.parse().unwrap(),
            project_id: self.project_id.parse().unwrap(),
            description: self.description,
            price: self.price,
            last_modified: self.last_modified,
            last_changer: self.last_changer,
            time: self.time,
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
            price: self.price,
            last_modified: self.last_modified,
            last_changer: self.last_changer,
            time: self.time,
        }
    }
}

impl Entity for AuditRequest<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}
