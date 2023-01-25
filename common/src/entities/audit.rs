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
pub struct Message {
    text: String,
    sender: ObjectId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Audit {
    pub id: ObjectId,
    pub customer_id: ObjectId,
    pub auditor_id: ObjectId,
    pub project_id: ObjectId,
    pub status: String,
    pub last_modified: NaiveDateTime,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub price: String,
    pub comment: String,
    pub visibility: Visibility,
}

impl Audit {
    pub fn to_view(self, name: String) -> View {
        View {
            id: self.id,
            name,
            description: self.comment,
            last_modified: self.last_modified,
            source: Source::Audit,
        }
    }
}

impl ToSchema for Audit {
    fn schema() -> Schema {
        ObjectBuilder::new()
            .property("id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("id")
            .property(
                "customer_id",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("customer_id")
            .property(
                "auditor_id",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("auditor_id")
            .property(
                "project_id",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("project_id")
            .property(
                "terms",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("terms")
            .property(
                "status",
                ObjectBuilder::new().schema_type(SchemaType::Object),
            )
            .required("status")
            .property(
                "last_modified",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("last_modified")
            .property(
                "issues",
                ObjectBuilder::new().schema_type(SchemaType::Array),
            )
            .required("issues")
            .property(
                "visibility",
                ObjectBuilder::new().schema_type(SchemaType::Array),
            )
            .required("visibility")
            .into()
    }
}
