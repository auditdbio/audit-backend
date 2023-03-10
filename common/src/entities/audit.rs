use std::{collections::HashMap, str::FromStr};

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repository::{Entity, TaggableEntity};

use super::view::{Source, View};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub struct Audit<Id> {
    pub id: Id,
    pub customer_id: Id,
    pub auditor_id: Id,
    pub project_id: Id,
    pub project_name: String,
    pub status: String,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub scope: Vec<String>,
    pub price: String,
    pub report_link: Option<String>,
    pub time_frame: String,
    pub tags: Vec<String>,
    pub last_modified: i64,
}

impl Audit<String> {
    pub fn parse(self) -> Audit<ObjectId> {
        Audit {
            id: ObjectId::from_str(&self.id).unwrap(),
            customer_id: ObjectId::from_str(&self.customer_id).unwrap(),
            auditor_id: ObjectId::from_str(&self.auditor_id).unwrap(),
            project_id: ObjectId::from_str(&self.project_id).unwrap(),
            project_name: self.project_name,
            status: self.status,
            auditor_contacts: self.auditor_contacts,
            customer_contacts: self.customer_contacts,
            scope: self.scope,
            price: self.price,
            report_link: self.report_link,
            time_frame: self.time_frame,
            tags: self.tags,
            last_modified: self.last_modified,
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
            status: self.status,
            auditor_contacts: self.auditor_contacts,
            customer_contacts: self.customer_contacts,
            scope: self.scope,
            price: self.price,
            report_link: self.report_link,
            time_frame: self.time_frame,
            tags: self.tags,
            last_modified: self.last_modified,
        }
    }
}

impl Audit<ObjectId> {
    pub fn to_view(self, name: String) -> View<ObjectId> {
        View {
            id: self.id,
            name,
            last_modified: self.last_modified,
            source: Source::Audit,
        }
    }
}

impl Entity for Audit<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

impl TaggableEntity for Audit<ObjectId> {
    fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}
