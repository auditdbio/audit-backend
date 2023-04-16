use std::collections::HashMap;

use common::repository::Entity;
use common::services::{AUDITORS_SERVICE, AUDITS_SERVICE, CUSTOMERS_SERVICE};
use lazy_static::lazy_static;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref PROTOCOL: String = "https".to_string();
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Since {
    pub id: ObjectId,
    pub name: String,
    pub dict: HashMap<String, i64>,
}

impl Entity for Since {
    fn id(&self) -> ObjectId {
        self.id
    }
}

impl Default for Since {
    fn default() -> Self {
        Self {
            id: ObjectId::new(),
            name: "since".to_string(),
            dict: {
                let mut map = HashMap::new();
                map.insert(
                    format!(
                        "{}://{}/api/project",
                        PROTOCOL.as_str(),
                        CUSTOMERS_SERVICE.as_str()
                    ),
                    0,
                );
                map.insert(
                    format!(
                        "{}://{}/api/auditor",
                        PROTOCOL.as_str(),
                        AUDITORS_SERVICE.as_str()
                    ),
                    0,
                );
                map.insert(
                    format!(
                        "{}://{}/api/customer",
                        PROTOCOL.as_str(),
                        CUSTOMERS_SERVICE.as_str()
                    ),
                    0,
                );
                map.insert(
                    format!(
                        "{}://{}/api/audit",
                        PROTOCOL.as_str(),
                        AUDITS_SERVICE.as_str()
                    ),
                    0,
                );
                map.insert(
                    format!(
                        "{}://{}/api/request",
                        PROTOCOL.as_str(),
                        AUDITS_SERVICE.as_str()
                    ),
                    0,
                );
                map
            },
        }
    }
}
