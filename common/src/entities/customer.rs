use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{ObjectBuilder, SchemaType},
    ToSchema,
};

use crate::repository::Entity;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Customer {
    pub user_id: ObjectId,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub contacts: HashMap<String, String>,
    pub tax: String,
    pub tags: Vec<String>,
}

impl<'s> ToSchema<'s> for Customer {
    fn schema() -> (
        &'s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "Customer",
            ObjectBuilder::new()
                .property(
                    "user_id",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
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
                .property(
                    "company",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .required("company")
                .property(
                    "contacts",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("contacts")
                .property(
                    "tax",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .required("tax")
                .property(
                    "tags",
                    ObjectBuilder::new().schema_type(SchemaType::Array),
                )
                .required("tags")
                .into(),
        )
    }
}

impl Entity for Customer {
    fn id(&self) -> ObjectId {
        self.user_id.clone()
    }
}
