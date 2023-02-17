use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, openapi::{Schema, ObjectBuilder, SchemaType}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ObjectId,
    pub customer_id: ObjectId,
    pub name: String,
    pub description: String,
    pub scope: Vec<String>,
    pub tags: Vec<String>,
    pub status: String,
}

impl<'s>  ToSchema<'s> for Project {
    fn schema() -> (&'s str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
        ("Project", ObjectBuilder::new()
            .property("id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("id")
            .property("customer_id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("customer_id")
            .property("name", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("name")
            .property("description", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("description")
            .property("scope", ObjectBuilder::new().schema_type(SchemaType::Array))
            .required("scope")
            .property("tags", ObjectBuilder::new().schema_type(SchemaType::Array))          
            .required("tags")
            .property("status", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("status")
            .into()
        )
    }
}
