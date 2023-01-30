use std::collections::HashMap;

use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{ObjectBuilder, Schema, SchemaType},
    ToSchema,
};

use super::view::{Source, View};

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

impl ToSchema for AuditRequest {
    fn schema() -> Schema {
        ObjectBuilder::new()
            .property("id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("id")
            .property(
                "auditor_id",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("auditor_id")
            .property(
                "customer_id",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("customer_id")
            .property(
                "project_id",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("project_id")
            .property(
                "auditor_contacts",
                ObjectBuilder::new().schema_type(SchemaType::Object),
            )
            .required("auditor_contacts")
            .property(
                "customer_contacts",
                ObjectBuilder::new().schema_type(SchemaType::Object),
            )
            .required("customer_contacts")
            .property(
                "comment",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("comment")
            .property(
                "price",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("price")
            .property(
                "last_modified",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .into()
    }
}
