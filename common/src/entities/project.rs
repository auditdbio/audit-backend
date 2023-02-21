use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{ObjectBuilder, SchemaType},
    ToSchema,
};

use crate::repository::{Entity, TaggableEntity};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: ObjectId,
    pub customer_id: ObjectId,
    pub name: String,
    pub description: String,
    pub scope: Vec<String>,
    pub tags: Vec<String>,
    pub status: String,
}

impl Entity for Project {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

impl TaggableEntity for Project {
    fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}

impl<'s> ToSchema<'s> for Project {
    fn schema() -> (
        &'s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "Project",
            ObjectBuilder::new()
                .property("id", ObjectBuilder::new().schema_type(SchemaType::Object))
                .required("id")
                .property(
                    "customer_id",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("customer_id")
                .property("name", ObjectBuilder::new().schema_type(SchemaType::String))
                .required("name")
                .property(
                    "description",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .required("description")
                .property("scope", ObjectBuilder::new().schema_type(SchemaType::Array))
                .required("scope")
                .property("tags", ObjectBuilder::new().schema_type(SchemaType::Array))
                .required("tags")
                .property(
                    "status",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .required("status")
                .into(),
        )
    }
}
