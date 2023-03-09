use std::str::FromStr;

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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct View<Id> {
    pub id: Id,
    pub name: String,
    pub source: Source,
    pub last_modified: NaiveDateTime,
}

impl View<String> {
    fn parse(self) -> View<ObjectId> {
        View {
            id: ObjectId::from_str(&self.id).unwrap(),
            name: self.name,
            source: self.source,
            last_modified: self.last_modified,
        }
    }
}

impl View<ObjectId> {
    fn serialize(self) -> View<String> {
        View {
            id: self.id.to_hex(),
            name: self.name,
            source: self.source,
            last_modified: self.last_modified,
        }
    }
}
