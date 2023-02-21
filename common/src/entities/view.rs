use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{ObjectBuilder, SchemaType},
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
    pub last_modified: NaiveDateTime,
}

impl<'s> ToSchema<'s> for View {
    fn schema() -> (
        &'s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "View",
            ObjectBuilder::new()
                .property("id", ObjectBuilder::new().schema_type(SchemaType::Object))
                .required("id")
                .property("name", ObjectBuilder::new().schema_type(SchemaType::String))
                .required("name")
                .property(
                    "source",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .required("source")
                .property(
                    "last_modified",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .required("last_modified")
                .into(),
        )
    }
}
