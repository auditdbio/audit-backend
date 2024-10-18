use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicReport {
    pub file_id: String,
    pub report_name: String,
    pub is_draft: bool,
    pub verification_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReport {
    pub is_draft: Option<bool>,
}
