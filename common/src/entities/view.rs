use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{ObjectBuilder, Schema, SchemaType},
    ToSchema,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Source {
    Request,
    Audit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct View {
    pub id: ObjectId,
    pub name: String,
    pub source: Source,
    pub description: String,
    pub last_modified: NaiveDateTime,
}

impl ToSchema for View {
    fn schema() -> Schema {
        ObjectBuilder::new()
            .property("id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("id")
            .property("name", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("name")
            .property(
                "source",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("source")
            .property(
                "description",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("description")
            .property(
                "last_modified",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("last_modified")
            .into()
    }
}
