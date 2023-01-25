use std::str::FromStr;

use mongodb::bson::oid::ObjectId;

use crate::error::{Error, Result};

pub mod audit;
pub mod audit_request;

fn parse_id(id: &str) -> Result<ObjectId> {
    ObjectId::from_str(id).map_err(|e| Error::Custom(e.to_string()))
}
