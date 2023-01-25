use std::collections::HashMap;

use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use super::{view::{View, Source}};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditRequest {
    pub id: ObjectId,
    pub auditor_id: ObjectId,
    pub customer_id: ObjectId,
    pub project_id: ObjectId,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub comment: Option<String>,
    pub price: Option<String>,
    pub last_modified: NaiveDateTime,
}


impl AuditRequest {
    pub fn to_view(self, name: String) -> View {
        View {
            id: self.id,
            name,
            description: self.comment.unwrap_or("".to_string()),
            last_modified: self.last_modified,
            source: Source::Request,
        }
    } 
}