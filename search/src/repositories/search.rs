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

        let find_options = if let Some(sort_by) = &query.sort_by {
            let sort_order = query.sort_order.unwrap_or(1);
            let mut sort = doc! {
                sort_by.clone(): sort_order,
                "_id": -1,
            };

            if sort_by == "price" {
                let sort_field = if sort_order == 1 {
                    "price_range.to"
                } else {
                    "price_range.from"
                }
                .to_string();
                sort.insert(sort_field, sort_order);
            }

            if kind.contains(&"auditor".to_string()) {
                sort.insert("kind", 1);
            }

            let mut skip = (query.page - 1) * query.per_page;
            let mut limit = (query.per_page * query.pages.unwrap_or(1)) as i64;

            if query.page == 0 {
                skip = 0;
                limit = 1000;
            }

            Some(
                FindOptions::builder()
                    .sort(sort)
                    .skip(skip)
                    .limit(limit)
                    .build(),
            )
        } else {
            None
        };

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

        // if kind.contains(&"customer".to_string()) {
        //     let price_from = query.price_from.unwrap_or(0);
        //     let price_to = query.price_to.unwrap_or(i64::MAX);
        //     docs.push(doc! {
        //         "$or": [
        //             {
        //                 "price": {
        //                     "$gte": price_from,
        //                     "$lte": price_to,
        //                 },
        //             },
        //             {
        //                 "price_range.from": {
        //                     "$lte": price_to,
        //
        //                 },
        //                 "price_range.to": {
        //                     "$gte": price_from,
        //                 },
        //             },
        //
        //         ]
        //     });
        // }

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
