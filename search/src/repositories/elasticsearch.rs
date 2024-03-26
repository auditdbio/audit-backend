use common::error;
use elasticsearch::Elasticsearch;

use crate::service::search::SearchResult;

pub struct ElasticRepository {
    repo: Elasticsearch,
}

impl ElasticRepository {
    pub async fn search() -> error::Result<SearchResult> {
        todo!()
    }
}
