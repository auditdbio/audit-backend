use std::collections::HashMap;
use std::hash::Hash;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::Entity;

use super::{audit_request::TimeRange, contacts::Contacts, issue::Issue};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub struct Audit<Id: Eq + Hash> {
    pub id: Id,
    pub customer_id: Id,
    pub auditor_id: Id,
    pub project_id: Id,

    pub project_name: String,
    pub avatar: String,
    pub description: String,
    pub status: String,
    pub scope: Vec<String>,
    pub price: i64,

    pub auditor_contacts: Contacts,
    pub customer_contacts: Contacts,
    pub tags: Vec<String>,
    pub last_modified: i64,
    pub report: Option<String>,
    pub report_name: Option<String>,
    pub time: TimeRange,

    #[serde(default)]
    pub issues: HashMap<Id, Issue<Id>>,
}

impl Audit<String> {
    pub fn parse(self) -> Audit<ObjectId> {
        Audit {
            id: self.id.parse().unwrap(),
            customer_id: self.customer_id.parse().unwrap(),
            auditor_id: self.auditor_id.parse().unwrap(),
            project_id: self.project_id.parse().unwrap(),
            project_name: self.project_name,
            avatar: self.avatar,
            description: self.description,
            status: self.status,
            auditor_contacts: self.auditor_contacts,
            customer_contacts: self.customer_contacts,
            scope: self.scope,
            price: self.price,
            tags: self.tags,
            last_modified: self.last_modified,
            report: self.report,
            report_name: self.report_name,
            time: self.time,
            issues: Issue::parse_map(self.issues),
        }
    }
}

impl Audit<ObjectId> {
    pub fn stringify(self) -> Audit<String> {
        Audit {
            id: self.id.to_hex(),
            customer_id: self.customer_id.to_hex(),
            auditor_id: self.auditor_id.to_hex(),
            project_id: self.project_id.to_hex(),
            project_name: self.project_name,
            avatar: self.avatar,
            description: self.description,
            status: self.status,
            auditor_contacts: self.auditor_contacts,
            customer_contacts: self.customer_contacts,
            scope: self.scope,
            price: self.price,
            tags: self.tags,
            last_modified: self.last_modified,
            report: self.report,
            report_name: self.report_name,
            time: self.time,
            issues: Issue::to_string_map(self.issues),
        }
    }
}

impl Entity for Audit<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}
