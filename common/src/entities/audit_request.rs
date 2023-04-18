use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::Entity;

use super::role::Role;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
pub struct PriceRange {
    pub from: i64,
    pub to: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
pub struct TimeRange {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub struct AuditRequest<Id> {
    pub id: Id,
    pub auditor_id: Id,
    pub customer_id: Id,
    pub project_id: Id,

    pub project_name: String,
    pub project_avatar: String,
    pub project_scope: Vec<String>,
    pub price_range: Option<PriceRange>,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,

    pub description: String,

    pub price: Option<i64>,
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
            project_name: self.project_name,
            project_avatar: self.project_avatar,
            description: self.description,
            auditor_contacts: self.auditor_contacts,
            customer_contacts: self.customer_contacts,
            project_scope: self.project_scope,
            price: self.price,
            last_modified: self.last_modified,
            last_changer: self.last_changer,
            time: self.time,
            price_range: self.price_range,
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
            project_name: self.project_name,
            project_avatar: self.project_avatar,
            description: self.description,
            auditor_contacts: self.auditor_contacts,
            customer_contacts: self.customer_contacts,
            project_scope: self.project_scope,
            price: self.price,
            last_modified: self.last_modified,
            last_changer: self.last_changer,
            time: self.time,
            price_range: self.price_range,
        }
    }
}

// impl AuditRequest<ObjectId> {
//     pub fn to_view(self, name: String) -> View<ObjectId> {
//         View {
//             id: self.id,
//             name,
//             last_modified: self.last_modified,
//             source: Source::Request,
//         }
//     }
// }

impl Entity for AuditRequest<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}
