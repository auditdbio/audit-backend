use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use utoipa::{openapi::{Schema, ObjectBuilder, SchemaType}, ToSchema};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    text: String,
    sender: ObjectId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    messages: Vec<Message>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Audit {
    pub id: ObjectId,
    pub customer_id: ObjectId,
    pub auditor_id: ObjectId,
    pub project_id: ObjectId,
    pub terms: String,
    pub status: String,
    pub last_modified: NaiveDateTime,
    pub issues: Vec<Issue>,
    pub visibility: Vec<ObjectId>,
}

impl ToSchema for Audit {
    fn schema() -> Schema {
        ObjectBuilder::new()
            .property("id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("id")
            .property("customer_id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("customer_id")
            .property("auditor_id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("auditor_id")
            .property("project_id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("project_id")
            .property("terms", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("terms")
            .property("status", ObjectBuilder::new().schema_type(SchemaType::Object))
            .required("status")
            .property("last_modified", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("last_modified")
            .property("issues", ObjectBuilder::new().schema_type(SchemaType::Array))
            .required("issues")
            .property("visibility", ObjectBuilder::new().schema_type(SchemaType::Array))
            .required("visibility")
            .into()
    }
}
