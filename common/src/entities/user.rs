use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use utoipa::{ToSchema, openapi::{Schema, ObjectBuilder, SchemaType}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: ObjectId,
    pub email: String,
    pub password: String,
    pub name: String,
}

impl ToSchema for User {
    fn schema() -> Schema {
        ObjectBuilder::new()
            .property("id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("id")
            .property("email", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("email")
            .property("password", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("password")
            .property("name", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("name")
            .into()
    }
}
