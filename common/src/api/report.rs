use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicReport {
    pub path: String,
    pub verification_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReport {
    pub is_draft: Option<bool>,
}
