use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Contacts {
    pub email: Option<String>,
    pub telegram: Option<String>,
    pub public_contacts: bool,
}
