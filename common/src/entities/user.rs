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
    pub current_role: String,
}

impl<'s> ToSchema<'s> for User {
    fn schema() -> (&'s str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
        ("User", ObjectBuilder::new()
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
        )
    }
}
