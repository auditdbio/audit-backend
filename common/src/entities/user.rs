use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{ObjectBuilder, Schema, SchemaType},
    ToSchema,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: ObjectId,
    pub email: String,
    pub password: String,
    pub name: String,
    pub required_account_type: String,
}

impl ToSchema for User {
    fn schema() -> Schema {
        ObjectBuilder::new()
            .property("id", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("id")
            .property(
                "email",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("email")
            .property(
                "password",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .required("password")
            .property("name", ObjectBuilder::new().schema_type(SchemaType::String))
            .required("name")
            .into()
    }
}
