use mongodb::bson::{doc, Bson, Document, oid::ObjectId};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use common::{
    error,
    repository::{Entity ,mongo_repository::MongoRepository},
    services::{API_PREFIX, AUDITORS_SERVICE, CUSTOMERS_SERVICE, PROTOCOL, USERS_SERVICE},
};

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
                map.insert(
                    format!(
                        "{}://{}/{}/organization/data",
                        PROTOCOL.as_str(),
                        USERS_SERVICE.as_str(),
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
