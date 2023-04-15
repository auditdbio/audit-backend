use std::collections::HashMap;

use mongodb::bson::{self, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::Entity;

use super::audit_request::TimeRange;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub struct Audit<Id> {
    pub id: Id,
    pub customer_id: Id,
    pub auditor_id: Id,
    pub project_id: Id,

    pub project_name: String,
    pub avatar: String,
    pub description: Option<String>,
    pub status: String,
    pub scope: Vec<String>,
    pub price: i64,


    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub report_link: Option<String>,
    pub tags: Vec<String>,
    pub last_modified: i64,
    pub report: Option<String>,
    pub time: TimeRange,
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
            report_link: self.report_link,
            tags: self.tags,
            last_modified: self.last_modified,
            report: self.report,
            time: self.time,
        }
    }

    pub fn to_doc(self) -> Document {
        let mut document = bson::to_document(&self).unwrap();
        document.insert("kind", "audit");
        document
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
            report_link: self.report_link,
            tags: self.tags,
            last_modified: self.last_modified,
            report: self.report,
            time: self.time,
        }
    }
}

// impl Audit<ObjectId> {
//     pub fn to_view(self, name: String) -> View<ObjectId> {
//         View {
//             id: self.id,
//             name,
//             last_modified: self.last_modified,
//             source: Source::Audit,
//         }
//     }
// }

impl Entity for Audit<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }

    fn timestamp(&self) -> i64 {
        self.last_modified
    }
}
