use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::repository::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Letter {
    pub id: ObjectId,
    pub email: String,
    pub message: String,
    pub subject: String,
    pub sender: Option<String>,
}

impl Entity for Letter {
    fn id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateLetter {
    pub recipient_id: Option<ObjectId>,
    pub recipient_name: Option<String>,
    pub email: String,
    pub message: String,
    pub subject: String,
}
