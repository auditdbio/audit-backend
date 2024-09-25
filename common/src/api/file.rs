use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeFile {
    pub private: Option<bool>,
    pub access_code: Option<String>,
}