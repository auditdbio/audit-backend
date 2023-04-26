use std::sync::Arc;

use common::repository::mongo_repository::MongoRepository;
use futures::StreamExt;
use mongodb::{
    bson::{doc, Bson, Document},
    options::{FindOptions, UpdateOptions},
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
        for doc in query.iter() {
            self.0
                .collection
                .update_one(
                    doc! {
                        "kind": doc.get("kind").unwrap(),
                        "id": doc.get("id").unwrap(),
                    },
                    doc! {
                        "$set": doc,
                    },
                    UpdateOptions::builder().upsert(true).build(),
                )
                .await?;
        }
        Ok(())
    }

    pub async fn search(&self, mut query: SearchQuery) -> anyhow::Result<Vec<Document>> {
        let find_options = if let Some(sort_by) = query.sort_by {
            let sort_order = query.sort_order.unwrap_or(1);
            let mut sort = doc! {
                sort_by.clone(): sort_order,
            };

            if &sort_by == "price" {
                let sort_field = if sort_order == 1 {
                    "price_range.to"
                } else {
                    "price_range.from"
                }
                .to_string();
                sort.insert(sort_field, sort_order);
            }
            Some(
                FindOptions::builder()
                    .sort(sort)
                    .build(),
            )
        } else {
            None
        };

        query.query = query.query.to_ascii_lowercase();

        let mut docs = Vec::new();

        if let Some(kind) = query.kind.clone() {
            docs.push(doc! {
                "kind": kind,
            });
        }

        if !query.query.is_empty() {
            let text = query.query;
            docs.push(doc! {
                "$text": {
                    "$search": text,
                },
            });
        }

        let tags = query
            .tags
            .split(" ")
            .filter_map(|s| {
                if !s.is_empty() {
                    Some(s.to_ascii_lowercase())
                } else {
                    None
                } // insensitive
            })
            .collect::<Vec<_>>();

        if !tags.is_empty() {
            docs.push(doc! {
                "search_tags": {
                    "$all": tags,
                },
            });
        }

        if &query.kind != &Some("customer".to_string()) {
        let price_from = query.price_from.unwrap_or(0);
        let price_to = query.price_to.unwrap_or(i64::MAX);
        docs.push(doc! {
            "$or": [
                {
                    "price": {
                        "$gte": price_from,
                        "$lte": price_to,
                    },
                },
                {
                    "price_range.from": {
                        "$lte": price_to,

                    },
                    "price_range.to": {
                        "$gte": price_from,
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

        docs.push(doc! {
            "$or": [
                {
                    "private": false,
                },
                {
                    "private": Bson::Null,
                }
            ]
        });

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
