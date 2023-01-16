use std::collections::HashMap;

use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]

pub struct Project {
    pub id: ObjectId,
    pub customer_id: ObjectId,
    pub name: String,
    pub description: String,
    pub git_url: String,
    pub git_folders: HashMap<String, String>,
    pub tags: Vec<String>,
    pub status: String,
}
