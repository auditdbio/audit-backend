use std::sync::Arc;

use common::repository::mongo_repository::MongoRepository;
use common::repository::{Entity, Repository};
use common::services::{AUDITORS_SERVICE, CUSTOMERS_SERVICE, AUDITS_SERVICE};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Since {
    pub id: ObjectId,
    pub service_origin: String,
    pub service_name: String,
    pub resource: String,
    pub since: i64,
}

impl Entity for Since {
    fn id(&self) -> ObjectId {
        ObjectId::new()
    }

    fn timestamp(&self) -> i64 {
        unreachable!()
    }
}

pub struct SinceRepo(Arc<MongoRepository<Since>>);

impl Clone for SinceRepo {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl SinceRepo {
    pub async fn new(mongo_uri: String) -> Self {
        let repo = MongoRepository::new(&mongo_uri, "search", "since").await;
        Self(Arc::new(repo))
    }

    pub async fn insert_default(&self) {
        let count = self.0.collection.count_documents(None, None).await.unwrap();

        let services = vec![
            Since {
                id: ObjectId::new(),
                service_name: "auditors".to_string(),
                service_origin: AUDITORS_SERVICE.to_string(),
                resource: "auditor".to_string(),
                since: 0,
            },
            Since {
                id: ObjectId::new(),
                service_name: "customers".to_string(),
                service_origin: CUSTOMERS_SERVICE.to_string(),
                resource: "customer".to_string(),
                since: 0,
            },

            Since {
                id: ObjectId::new(),
                service_name: "customers".to_string(),
                service_origin: CUSTOMERS_SERVICE.to_string(),
                resource: "project".to_string(),
                since: 0,
            },

            Since {
                id: ObjectId::new(),
                service_name: "audits".to_string(),
                service_origin: AUDITS_SERVICE.to_string(),
                resource: "audit".to_string(),
                since: 0,
            },


            Since {
                id: ObjectId::new(),
                service_name: "audits".to_string(),
                service_origin: AUDITS_SERVICE.to_string(),
                resource: "request".to_string(),
                since: 0,
            },
        ];

        if count == services.len() as u64 {
            return;
        }

        for service in services {
            self.0.create(&service).await.unwrap();
        }
    }

    pub async fn get_all(&self) -> Result<Vec<Since>, mongodb::error::Error> {
        self.0.find_all(0, 100).await
    }

    pub async fn update(&self, since: Since) -> Result<(), mongodb::error::Error> {
        self.0.delete("id", &since.id).await?;
        self.0.create(&since).await?;
        Ok(())
    }
}
