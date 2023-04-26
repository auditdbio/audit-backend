use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::repository::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Letter {
    pub id: ObjectId,
    pub email: String,
    pub message: String,
    pub subject: String,
}

impl Entity for Letter {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLetter {
    pub email: String,
    pub message: String,
    pub subject: String,
}
