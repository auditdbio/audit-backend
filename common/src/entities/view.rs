use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum Source {
    Request,
    Audit,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct View {
    pub id: ObjectId,
    pub name: String,
    pub source: Source,
    pub description: String,
    pub last_modified: NaiveDateTime,
    
}