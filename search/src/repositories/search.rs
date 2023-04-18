use std::sync::Arc;

use common::repository::mongo_repository::MongoRepository;
use futures::StreamExt;
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
    IndexModel,
};

use crate::service::search::SearchQuery;

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

    pub async fn insert(&self, query: Vec<Document>) -> anyhow::Result<()> {
        self.0.collection.insert_many(query, None).await?;
        Ok(())
    }

    pub async fn search(&self, query: SearchQuery) -> anyhow::Result<Vec<Document>> {
        let find_options = if let Some(sort_by) = query.sort_by {
            Some(
                FindOptions::builder()
                    .sort(doc! {
                        sort_by: query.sort_order.unwrap_or(1),
                    })
                    .build(),
            )
        } else {
            None
        };

        let mut docs = Vec::new();

        if let Some(kind) = query.kind {
            docs.push(doc! {
                "kind": kind,
            });
        }

        if !query.query.is_empty() || !query.tags.is_empty() {
            let text = query.query + " " + &query.tags;
            docs.push(doc! {
                "$text": {
                    "$search": text,
                },
            });
        }

        if let (Some(price_from), Some(price_to)) = (query.price_from, query.price_to) {
            docs.push(doc! {
                "$or": [
                    {
                        "price": {
                            "$gte": price_from,
                            "$lte": price_to,
                        },
                    },
                    {
                        "price_range": {
                            "begin": {
                                "$gte": price_from,
                            },
                            "end": {
                                "$lte": price_to,
                            },
                        },
                    },

                ]
            });
        }

        if let (Some(time_from), Some(time_to)) = (query.time_from, query.time_to) {
            docs.push(doc! {
                "time": {
                    "$gte": time_from,
                    "$lte": time_to,
                }
            });
           
        }

        if let Some(ready_to_wait) = query.ready_to_wait {
            docs.push(doc! {
                "ready_to_wait": ready_to_wait,
            });
        }

        log::info!("Search query: {:?}", docs);

        let cursor = self
            .0
            .collection
            .find(doc! { "$and": docs}, find_options)
            .await
            .unwrap();

        Ok(cursor
            .collect::<Vec<Result<Document, _>>>()
            .await
            .into_iter()
            .map(|x| x.unwrap())
            .collect())
    }
}
