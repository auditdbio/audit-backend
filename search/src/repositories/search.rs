use std::sync::Arc;

use common::repository::mongo_repository::MongoRepository;
use futures::StreamExt;
use mongodb::{
    bson::{doc, Bson, Document},
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

        let mut document = doc! {};

        if let Some(kind) = query.kind {
            document.insert("kind", Bson::String(kind));
        }

        if !query.query.is_empty() || !query.tags.is_empty() {
            let text = query.query + " " + &query.tags;
            document.insert(
                "$text",
                doc! {
                    "$search": text,
                },
            );
        }

        if let Some(price_range) = query.price {
            document.insert(
                "price",
                doc! {
                    "$gte": price_range.from,
                    "$lte": price_range.to,
                },
            );
            document.insert(
                "price_range",
                doc! {
                    "begin": {
                        "$gte": price_range.from,
                    },
                    "end": {
                        "$lte": price_range.to,
                    },
                },
            );
        }

        if let Some(time_range) = query.time {
            document.insert(
                "time",
                doc! {
                    "begin": {
                        "$gte": time_range.from,
                    },
                    "end": {
                        "$lte": time_range.to,
                    },
                },
            );
        }

        if let Some(ready_to_wait) = query.ready_to_wait {
            document.insert(
                "publish_options",
                doc! {
                    "ready_to_wait": doc! {
                        "$eq": ready_to_wait,
                    },
                },
            );
        }

        let cursor = self
            .0
            .collection
            .find(document, find_options)
            .await
            .unwrap();

        cursor
            .collect::<Vec<Result<Document, _>>>()
            .await
            .into_iter()
            .map(|x| x.unwrap())
            .collect()
    }
}
