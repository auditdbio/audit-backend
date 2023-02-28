use std::collections::HashMap;

use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{ObjectBuilder, Schema, SchemaType},
    ToSchema,
};

use crate::repository::Entity;

use super::{
    role::Role,
    view::{Source, View},
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
pub struct PriceRange {
    pub lower_bound: i64,
    pub upper_bound: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AuditRequest {
    pub id: ObjectId,
    pub auditor_id: ObjectId,
    pub customer_id: ObjectId,
    pub project_id: ObjectId,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub scope: Vec<String>,
    pub price: Option<i64>,
    pub price_range: Option<PriceRange>,
    pub time_frame: String,
    pub last_modified: NaiveDateTime,
    pub opener: Role,
}

impl AuditRequest {
    pub fn to_view(self, name: String) -> View {
        View {
            id: self.id,
            name,
            last_modified: self.last_modified,
            source: Source::Request,
        }
    }
}

impl<'s> ToSchema<'s> for AuditRequest {
    fn schema() -> (
        &'s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "AuditRequest",
            ObjectBuilder::new()
                .property("id", ObjectBuilder::new().schema_type(SchemaType::Object))
                .required("id")
                .property(
                    "auditor_id",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("auditor_id")
                .property(
                    "customer_id",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("customer_id")
                .property(
                    "project_id",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("project_id")
                .property(
                    "auditor_contacts",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("auditor_contacts")
                .property(
                    "customer_contacts",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("customer_contacts")
                .property("scope", ObjectBuilder::new().schema_type(SchemaType::Array))
                .required("scope")
                .property(
                    "price",
                    ObjectBuilder::new().schema_type(SchemaType::Integer),
                )
                .required("price")
                .property(
                    "price_range",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("price_range")
                .property(
                    "last_modified",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .property("role", ObjectBuilder::new().schema_type(SchemaType::String))
                .into(),
        )
    }
}

impl Entity for AuditRequest {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}
