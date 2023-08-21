use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::default_timestamp;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Substitution {
    pub text: String,
    pub styles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationInner {
    pub message: String,
    pub title: Option<String>,
    #[serde(default)]
    pub substitutions: Vec<Substitution>,
    pub is_read: bool,
    pub is_sound: bool,
    #[serde(default)]
    pub role: String,

    #[serde(default)]
    pub links: Vec<String>,

    #[serde(default = "default_timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateNotification {
    pub user_id: ObjectId,
    pub inner: NotificationInner,
}
