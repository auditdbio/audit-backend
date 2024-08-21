use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicReport {
    pub path: String,
    pub verification_code: Option<String>,
}
