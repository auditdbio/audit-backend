use std::{collections::HashMap, str::FromStr};

use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    ToSchema,
};

use crate::repository::Entity;

use super::{
    role::Role,
    view::{Source, View},
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
pub struct PriceRange {
    pub lower_bound: String,
    pub upper_bound: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub struct AuditRequest<Id> {
    pub id: Id,
    pub auditor_id: Id,
    pub customer_id: Id,
    pub project_id: Id,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub scope: Vec<String>,
    pub price: Option<String>,
    pub time_frame: String,
    pub last_modified: i64,
    pub opener: Role,
}

impl AuditRequest<String> {
    pub fn parse(self) -> AuditRequest<ObjectId> {
        AuditRequest {
            id: ObjectId::from_str(&self.id).unwrap(),
            auditor_id: ObjectId::from_str(&self.auditor_id).unwrap(),
            customer_id: ObjectId::from_str(&self.customer_id).unwrap(),
            project_id: ObjectId::from_str(&self.project_id).unwrap(),
            auditor_contacts: self.auditor_contacts,
            customer_contacts: self.customer_contacts,
            scope: self.scope,
            price: self.price,
            time_frame: self.time_frame,
            last_modified: self.last_modified,
            opener: self.opener,
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
            auditor_contacts: self.auditor_contacts,
            customer_contacts: self.customer_contacts,
            scope: self.scope,
            price: self.price,
            time_frame: self.time_frame,
            last_modified: self.last_modified,
            opener: self.opener,
        }
    }
}

impl AuditRequest<ObjectId> {
    pub fn to_view(self, name: String) -> View<ObjectId> {
        View {
            id: self.id,
            name,
            last_modified: self.last_modified,
            source: Source::Request,
        }
    }
}

impl Entity for AuditRequest<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}
