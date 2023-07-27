use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMessage {
    pub id: String,
    pub from: String,
    pub group: String,
    pub time: i64,
    pub text: String,
}
