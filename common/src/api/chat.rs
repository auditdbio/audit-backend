use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMessage {
    pub from: String,
    pub to: String,
    pub time: i64,
    pub text: String,
}
