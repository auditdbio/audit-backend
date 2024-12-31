use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use chrono::Utc;

use common::{error, impl_has_last_modified, default_timestamp};
use common::repository::mongo_repository::MongoRepository;
use common::repository::{Entity, HasLastModified};
use common::services::{API_PREFIX, AUDITORS_SERVICE, CUSTOMERS_SERVICE, PROTOCOL};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Bson, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Since {
    pub id: ObjectId,
    pub name: String,
    pub dict: HashMap<String, i64>,
    #[serde(default = "default_timestamp")]
    pub last_modified: i64,
}

impl_has_last_modified!(Since);

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
            last_modified: Utc::now().timestamp_micros(),
            dict: {
                let mut map = HashMap::new();
                map.insert(
                    format!(
                        "{}://{}/{}/project/data",
                        PROTOCOL.as_str(),
                        CUSTOMERS_SERVICE.as_str(),
                        API_PREFIX.as_str(),
                    ),
                    0,
                );
                map.insert(
                    format!(
                        "{}://{}/{}/badge/data",
                        PROTOCOL.as_str(),
                        AUDITORS_SERVICE.as_str(),
                        API_PREFIX.as_str(),
                    ),
                    0,
                );
                map.insert(
                    format!(
                        "{}://{}/{}/auditor/data",
                        PROTOCOL.as_str(),
                        AUDITORS_SERVICE.as_str(),
                        API_PREFIX.as_str(),
                    ),
                    0,
                );
                map.insert(
                    format!(
                        "{}://{}/{}/customer/data",
                        PROTOCOL.as_str(),
                        CUSTOMERS_SERVICE.as_str(),
                        API_PREFIX.as_str(),
                    ),
                    0,
                );

                map
            },
        }
    }
}

#[derive(Clone)]
pub struct SinceRepo {
    repo: Arc<MongoRepository<Since>>,
}

impl SinceRepo {
    pub fn new(repo: MongoRepository<Since>) -> Self {
        Self {
            repo: Arc::new(repo),
        }
    }

    pub async fn update(&self, dict: HashMap<String, i64>) -> error::Result<()> {
        let doc: Document = dict.into_iter().map(|(k, v)| (k, Bson::Int64(v))).collect();
        self.repo
            .collection
            .update_one(
                doc! {"name": "since"},
                doc! {
                    "$set": {
                        "dict": doc
                    }
                },
                None,
            )
            .await?;
        Ok(())
    }
}

impl Deref for SinceRepo {
    type Target = MongoRepository<Since>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
