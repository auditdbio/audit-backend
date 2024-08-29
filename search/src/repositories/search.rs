use std::sync::Arc;

use common::{error, repository::mongo_repository::MongoRepository};
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    options::{FindOptions, UpdateOptions},
    IndexModel,
};

use crate::service::search::{SearchQuery, SearchResult};

#[derive(Clone)]
pub struct SearchRepo(Arc<(MongoRepository<Document>, MongoRepository<Document>)>);

impl SearchRepo {
    pub async fn new(mongo_uri: String) -> Self {
        let repo = MongoRepository::new(&mongo_uri, "search", "queries").await;
        let trash = MongoRepository::new(&mongo_uri, "search", "trash").await;
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
        Self(Arc::new((repo, trash)))
    }

    pub async fn insert(&self, query: Vec<Document>) -> error::Result<()> {
        for doc in query.iter() {
            self.0
                 .0
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

    pub async fn search(&self, query: &SearchQuery) -> error::Result<SearchResult> {
        let kind = query
            .kind
            .clone()
            .unwrap_or(String::new())
            .split(' ')
            .filter_map(|s| {
                if !s.is_empty() {
                    Some(s.to_ascii_lowercase())
                } else {
                    None
                } // insensitive
            })
            .collect::<Vec<_>>();

        let mut skip = (query.page - 1) * query.per_page;
        let mut limit = (query.per_page * query.pages.unwrap_or(1)) as i64;

        if query.page == 0 {
            skip = 0;
            limit = 1000;
        }

        let sort_order = query.sort_order.unwrap_or(1);
        let mut sort = doc! {};

        if kind.contains(&"auditor".to_string()) {
            sort.insert("kind", 1);
        }

        if let Some(sort_by) = &query.sort_by {
            if sort_by == "price" {
                if kind.contains(&"auditor".to_string()) {
                    sort.insert(
                        "price_range.to",
                        doc! { "$ifNull": ["$price_range.to", 0],
                            "$sort": sort_order }
                    );
                    sort.insert(
                        "price_range.from",
                        doc! { "$ifNull": ["$price_range.from", 0],
                            "$sort": sort_order }
                    );
                } else if kind.contains(&"project".to_string()) {
                    sort.insert("price", doc! { "$ifNull": ["$price", 0], "$sort": sort_order });
                }
            } else if sort_by == "rating" && kind.contains(&"auditor".to_string()) {
                sort.insert("rating", doc! { "$ifNull": ["$rating", 0], "$sort": sort_order });
            }
        } else {
            sort.insert("_id", -1);
        }

        let find_options = FindOptions::builder()
            .sort(sort)
            .skip(skip)
            .limit(limit)
            .build();

        let mut docs = vec![
            doc! {
                "deleted": Bson::Null,
            },
            doc! {
                "$or": [
                    {
                        "private": false,
                    },
                    {
                        "private": Bson::Null,
                    }
                ]
            },
        ];

        if !kind.is_empty() {
            docs.push(doc! {
                "kind": {
                    "$in": kind.clone(),
                },
            });
        }

        if !query.query.is_empty() {
            docs.push(doc! {
                "$text": {
                    "$search": &query.query,
                },
            });
        }

        let tags = query
            .tags
            .split(' ')
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

        if !kind.contains(&"customer".to_string()) {
            if kind.contains(&"auditor".to_string()) || kind.contains(&"badge".to_string()) {
                docs.push(doc! {
                    "$and": [
                        {
                            "price_range.from": {
                                "$gte": query.price_from.unwrap_or(0),
                            },
                        },
                        {
                            "price_range.to": {
                                "$lte": query.price_to.unwrap_or(i64::MAX),
                            },
                        }
                    ]
                });
            } else if kind.contains(&"project".to_string()) {
                // TODO: add projects with total cost to the result
                docs.push(doc! {
                    "price": {
                        "$gte": query.price_from.unwrap_or(0),
                        "$lte": query.price_to.unwrap_or(i64::MAX),
                    }
                });
            }
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
        log::info!("Search options: {:?}", find_options);

        let result: Vec<Document> = self
            .0
             .0
            .collection
            .find(doc! { "$and": docs.clone()}, find_options)
            .await
            .unwrap()
            .collect::<Vec<Result<Document, _>>>()
            .await
            .into_iter()
            .map(|x| x.unwrap())
            .collect();

        let total_documents = self
            .0
             .0
            .collection
            .count_documents(doc! { "$and": docs}, None)
            .await
            .unwrap();

        log::info!("Search result: {:?}", serde_json::to_string_pretty(&result));

        Ok(SearchResult {
            result,
            total_documents,
        })
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<()> {
        let deleted = self
            .0
             .0
            .collection
            .find_one_and_delete(doc! {"id": id.to_hex()}, None)
            .await?;

        if let Some(deleted) = deleted {
            self.0 .1.collection.insert_one(deleted, None).await?;
        }
        Ok(())
    }
}
