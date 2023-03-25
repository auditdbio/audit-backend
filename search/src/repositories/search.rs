use std::sync::Arc;

use common::repository::mongo_repository::MongoRepository;
use futures::StreamExt;
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
    IndexModel,
};

use crate::SearchQuery;

#[derive(Clone)]
pub struct SearchRepo(Arc<MongoRepository<Document>>);

impl SearchRepo {
    pub async fn new(mongo_uri: String) -> Self {
        let repo = MongoRepository::new(&mongo_uri, "search", "queries").await;
        repo.collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! {
                        "$**": "text",
                    })
                    .build(),
                None,
            )
            .await
            .unwrap();
        Self(Arc::new(repo))
    }

    pub async fn insert(&self, query: Vec<Document>) {
        self.0.collection.insert_many(query, None).await.unwrap();
    }

    pub async fn find(&self, query: SearchQuery) -> Vec<Document> {
        let text = query.query + " " + &query.tags;

        // let find_options = FindOptions::builder()
        //     .sort(doc! {
        //         query.sort_by: query.sort_order,
        //     })
        //     .build();
        let find_options = None;

        let mut cursor = self
            .0
            .collection
            .find(
                doc! {
                    "kind": query.kind,
                    "$text": {
                        "$search": text
                    },
                },
                find_options,
            )
            .await
            .unwrap();

        let mut result = Vec::new();
        while let Some(doc) = cursor.next().await {
            let doc = doc.unwrap();
            // let tax_rate = doc.get_i64("tax_rate");
            // let time_from = doc.get_i64("time_from");
            // let time_to = doc.get_i64("time_to");
            let ready_to_wait = doc.get_bool("ready_to_wait");

            // if let Ok(tax_rate) = tax_rate {
            //     if tax_rate < query.tax_rate_from as i64 || tax_rate > query.tax_rate_to as i64 {
            //         continue;
            //     }
            // }

            // if let Ok(time_from) = time_from {
            //     if time_from < query.time_from as i64 {
            //         continue;
            //     }
            // }

            // if let Ok(time_to) = time_to {
            //     if time_to > query.time_to as i64 {
            //         continue;
            //     }
            // }

            if let Ok(ready_to_wait) = ready_to_wait {
                if ready_to_wait != query.ready_to_wait.unwrap_or(ready_to_wait) {
                    continue;
                }
            }

            result.push(doc);
        }
        result
    }
}
