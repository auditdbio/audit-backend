use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;

use super::role::Role;

pub struct AuditRequest {
    pub id: ObjectId,
    pub opener: Role,
    pub auditor_id: ObjectId,
    pub customer_id: ObjectId,
    pub project_id: ObjectId,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub comment: Option<String>,
    pub price: Option<String>,
}
