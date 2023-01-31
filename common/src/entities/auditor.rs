use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{ObjectBuilder, Schema, SchemaType},
    ToSchema,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auditor {
    pub user_id: ObjectId,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub tags: Vec<String>,
    pub contacts: HashMap<String, String>,
}

impl ToSchema for Auditor {
    fn schema() -> Schema {
        ObjectBuilder::new()
            .property(
                "user_id",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("user_id")
            .property(
                "first_name",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("first_name")
            .property(
                "last_name",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("last_name")
            .property(
                "about",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("about")
            .property("tags", ObjectBuilder::new().schema_type(SchemaType::Array))
            .required("tags")
            .property(
                "contacts",
                ObjectBuilder::new().schema_type(SchemaType::Object),
            )
            .required("contacts")
            .into()
    }
}
