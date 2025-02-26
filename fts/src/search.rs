use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    db::MongoDb,
    error::ServiceResult,
    index::SearchIndex,
    models::{SearchQuery, SearchResponse},
    storage::Storage,
};

pub struct SearchService {
    index: Arc<RwLock<SearchIndex>>,
    storage: Arc<Storage>,
    db: Arc<MongoDb>,
}

impl SearchService {
    pub async fn new(
        index_path: PathBuf,
        storage_path: PathBuf,
        mongo_uri: &str,
        db_name: &str,
    ) -> ServiceResult<Self> {
        let index = SearchIndex::open(&index_path).or_else(|_| SearchIndex::new(&index_path))?;
        let storage = Storage::new(&storage_path)?;
        let db = MongoDb::new(mongo_uri, db_name).await?;

        Ok(Self {
            index: Arc::new(RwLock::new(index)),
            storage: Arc::new(storage),
            db: Arc::new(db),
        })
    }

    pub async fn sync(&self) -> ServiceResult<()> {
        let last_sync = self.storage.get_last_sync_timestamp()?.unwrap_or(0);
        let auditors = self.db.get_auditors_since(last_sync).await?;

        if auditors.is_empty() {
            return Ok(());
        }

        let mut max_timestamp = last_sync;
        let mut index = self.index.write().await;

        for auditor in auditors {
            max_timestamp = max_timestamp.max(auditor.last_modified);
            index.index_auditor(&auditor)?;
        }

        index.commit()?;
        self.storage.set_last_sync_timestamp(max_timestamp)?;

        Ok(())
    }

    pub async fn cleanup(&self) -> ServiceResult<()> {
        let index = self.index.read().await;
        let query = SearchQuery {
            text: None,
            name: None,
            company: None,
            free_from: None,
            tags: None,
            price_range_from: None,
            price_range_to: None,
            rating_from: None,
            rating_to: None,
            sort: None,
            page: Some(1),
            per_page: Some(1000), // Process in batches of 1000
            partial_match: Some(false),
        };

        let (ids, _) = index.search(&query)?;
        let mut to_delete = Vec::new();

        for id in ids {
            if !self.db.check_auditor_exists(&id).await? {
                to_delete.push(id);
            }
        }

        if !to_delete.is_empty() {
            let mut index = self.index.write().await;
            for id in to_delete {
                index.delete_by_id(&id)?;
            }
            index.commit()?;
        }

        Ok(())
    }

    pub async fn search(&self, query: SearchQuery) -> ServiceResult<SearchResponse> {
        let index = self.index.read().await;
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(10);

        let (ids, total_documents) = index.search(&query)?;
        let auditors = self.db.get_auditors_by_ids(&ids).await?;

        Ok(SearchResponse {
            result: auditors,
            total_documents,
            page,
            per_page,
        })
    }

    pub async fn clear(&self) -> ServiceResult<()> {
        self.storage.clear()?;
        let mut index = self.index.write().await;
        index.commit()?;
        Ok(())
    }
}