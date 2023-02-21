use std::collections::HashMap;

use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{ObjectBuilder, SchemaType},
    ToSchema,
};

use crate::repository::Entity;

use super::view::{Source, View};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Audit {
    pub id: ObjectId,
    pub customer_id: ObjectId,
    pub auditor_id: ObjectId,
    pub project_id: ObjectId,
    pub status: String,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub scope: Vec<String>,
    pub price: i64,
    pub report_link: Option<String>,
    pub time_frame: String,
    pub last_modified: NaiveDateTime,
}

impl Audit {
    pub fn to_view(self, name: String) -> View {
        View {
            id: self.id,
            name,
            last_modified: self.last_modified,
            source: Source::Audit,
        }
    }
}

impl<'s> ToSchema<'s> for Audit {
    fn schema() -> (
        &'s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            "Audit",
            ObjectBuilder::new()
                .property("id", ObjectBuilder::new().schema_type(SchemaType::Object))
                .required("id")
                .property(
                    "customer_id",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("customer_id")
                .property(
                    "auditor_id",
                    ObjectBuilder::new().schema_type(SchemaType::Object),
                )
                .required("auditor_id")
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
                    "report_link",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .property(
                    "time_frame",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .required("time_frame")
                .property(
                    "status",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .required("status")
                .property(
                    "last_modified",
                    ObjectBuilder::new().schema_type(SchemaType::String),
                )
                .required("last_modified")
                .into(),
        )
    }
}

impl Entity for Audit {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}