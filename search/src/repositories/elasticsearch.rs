use common::error;
use elasticsearch::{CreateParts, Elasticsearch, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::service::search::SearchQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticSearchResult {
    #[serde(rename = "totalDocuments")]
    pub total_documents: u64,

    pub result: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct ElasticRepository {
    repo: Elasticsearch,
}

impl ElasticRepository {
    pub async fn search(&self, query: SearchQuery) -> error::Result<ElasticSearchResult> {
        let sort = if let Some(sort_by) = &query.sort_by {
            let sort_order = if query.sort_order == Some(1) {
                "asc"
            } else {
                "desc"
            };
            json! {[{sort_by: sort_order}]}
        } else {
            json! {[]}
        };

        let tags = query.tags.split_whitespace().collect::<Vec<_>>();
        let from = (query.page - 1) * query.per_page;
        let size = query.per_page;

        let result = self
            .repo
            .search(SearchParts::None)
            .body(json!({
                "from": from,
                "size": size,
                "query": {
                    "query_string": {
                        "query": query.query,
                        "fields": ["name", "description", "tags"]
                    },
                    "match": {
                        "kind": query.kind,
                        "tags": tags
                    }
                },
                "sort": sort
            }))
            .allow_no_indices(true)
            .send()
            .await?;

        let body = result.json::<serde_json::Value>().await?;
        let total = body["hits"]["total"]["value"].as_u64().unwrap_or(0);
        let hits = body["hits"]["hits"].as_array().unwrap_or(&vec![]).clone();

        Ok(ElasticSearchResult {
            total_documents: total,
            result: hits,
        })
    }

    pub async fn insert(&self, value: Value) -> error::Result<()> {
        let result = self
            .repo
            .create(CreateParts::IndexId("description", "id"))
            .body(&value)
            .send()
            .await?;
        self.repo
            .update(elasticsearch::UpdateParts::IndexId("description", "id"))
            .body(value)
            .send()
            .await?;
        Ok(())
    }

    pub async fn delete(&self) -> error::Result<()> {
        self.repo
            .delete(elasticsearch::DeleteParts::IndexId("description", "id"))
            .send()
            .await?;
        Ok(())
    }
}
