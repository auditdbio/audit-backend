use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    db::MongoDb,
    error::ServiceResult,
    index::SearchIndex,
    models::{SearchQuery, SearchResponse, EntityKind, SearchResult},
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
    ) -> ServiceResult<Self> {
        let index = SearchIndex::open(&index_path).or_else(|_| SearchIndex::new(&index_path))?;
        let storage = Storage::new(&storage_path)?;
        let db = MongoDb::new(mongo_uri).await?;

        Ok(Self {
            index: Arc::new(RwLock::new(index)),
            storage: Arc::new(storage),
            db: Arc::new(db),
        })
    }

    pub async fn sync(&self) -> ServiceResult<()> {
        let last_sync = self.storage.get_last_sync_timestamp()?.unwrap_or(0);

        let auditors = self.db.get_auditors_since(last_sync).await?;
        let badges = self.db.get_badges_since(last_sync).await?;
        let customers = self.db.get_customers_since(last_sync).await?;
        let projects = self.db.get_projects_since(last_sync).await?;

        tracing::info!("auditors: {}, badges: {}, customers: {}, projects: {}", auditors.len(), badges.len(), customers.len(), projects.len());

        if auditors.is_empty() && badges.is_empty() && customers.is_empty() && projects.is_empty() {
            return Ok(());
        }

        let mut max_timestamp = last_sync;
        let mut index = self.index.write().await;

        for auditor in auditors {
            max_timestamp = max_timestamp.max(auditor.last_modified);
            index.index_auditor(&auditor.stringify())?;
        }

        for badge in badges {
            max_timestamp = max_timestamp.max(badge.last_modified);
            index.index_badge(&badge.stringify())?;
        }

        for customer in customers {
            max_timestamp = max_timestamp.max(customer.last_modified);
            index.index_customer(&customer.stringify())?;
        }

        for project in projects {
            max_timestamp = max_timestamp.max(project.last_modified);
            index.index_project(&project.stringify())?;
        }

        index.commit()?;
        self.storage.set_last_sync_timestamp(max_timestamp)?;

        Ok(())
    }

    pub async fn cleanup(&self) -> ServiceResult<()> {
        let index = self.index.read().await;

        let entity_kinds = [
            EntityKind::Auditor,
            EntityKind::Badge,
            EntityKind::Customer,
            EntityKind::Project,
        ];
        
        let mut all_to_delete = Vec::new();

        for kind in &entity_kinds {
            let query = SearchQuery {
                text: None,
                name: None,
                company: None,
                free_from: None,
                tags: None,
                price_from: None,
                price_to: None,
                rating_from: None,
                rating_to: None,
                sort: None,
                page: Some(1),
                per_page: Some(1000), // Process in batches of 1000
                partial_match: Some(false),
                kind: vec![kind.clone()],
            };

            let (ids, _) = index.search(&query)?;
            let mut to_delete = Vec::new();

            for id in ids {
                if !self.db.check_entity_exists(kind, &id).await? {
                    to_delete.push((kind.clone(), id));
                }
            }
            
            all_to_delete.extend(to_delete);
        }

        if !all_to_delete.is_empty() {
            let mut index = self.index.write().await;
            for (_, id) in all_to_delete {
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

        let mut result = Vec::new();

        let kinds = &query.kind;

        let mut id_to_kind = std::collections::HashMap::new();

        for id in &ids {
            for kind in kinds {
                if self.db.check_entity_exists(kind, id).await.unwrap_or(false) {
                    id_to_kind.insert(id.clone(), kind.clone());
                    break;
                }
            }
        }

        let mut auditor_ids = Vec::new();
        let mut badge_ids = Vec::new();
        let mut customer_ids = Vec::new();
        let mut project_ids = Vec::new();
        
        for id in &ids {
            if let Some(kind) = id_to_kind.get(id) {
                match kind {
                    EntityKind::Auditor => auditor_ids.push(id.clone()),
                    EntityKind::Badge => badge_ids.push(id.clone()),
                    EntityKind::Customer => customer_ids.push(id.clone()),
                    EntityKind::Project => project_ids.push(id.clone()),
                }
            }
        }

        if !auditor_ids.is_empty() {
            let auditors = self.db.get_auditors_by_ids(&auditor_ids).await?;
            for auditor in auditors {
                result.push(SearchResult::Auditor(auditor.stringify()));
            }
        }
        
        if !badge_ids.is_empty() {
            let badges = self.db.get_badges_by_ids(&badge_ids).await?;
            for badge in badges {
                result.push(SearchResult::Badge(badge.stringify()));
            }
        }
        
        if !customer_ids.is_empty() {
            let customers = self.db.get_customers_by_ids(&customer_ids).await?;
            for customer in customers {
                result.push(SearchResult::Customer(customer.stringify()));
            }
        }
        
        if !project_ids.is_empty() {
            let projects = self.db.get_projects_by_ids(&project_ids).await?;
            for project in projects {
                result.push(SearchResult::Project(project.stringify()));
            }
        }

        result.sort_by(|a, b| {
            let a_id = match a {
                SearchResult::Auditor(auditor) => &auditor.user_id,
                SearchResult::Badge(badge) => &badge.user_id,
                SearchResult::Customer(customer) => &customer.user_id,
                SearchResult::Project(project) => &project.id,
            };
            
            let b_id = match b {
                SearchResult::Auditor(auditor) => &auditor.user_id,
                SearchResult::Badge(badge) => &badge.user_id,
                SearchResult::Customer(customer) => &customer.user_id,
                SearchResult::Project(project) => &project.id,
            };
            
            let a_pos = ids.iter().position(|id| id == a_id).unwrap_or(usize::MAX);
            let b_pos = ids.iter().position(|id| id == b_id).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos)
        });

        Ok(SearchResponse {
            result,
            total_documents,
            page,
            per_page,
        })
    }

    // pub async fn clear(&self) -> ServiceResult<()> {
    //     self.storage.clear()?;
    //     let mut index = self.index.write().await;
    //     index.clear()?;
    //     Ok(())
    // }

    pub async fn recreate_index(&self) -> ServiceResult<()> {
        self.storage.clear()?;
        let index_path = std::env::current_dir()?.join("data").join("index");
        if index_path.exists() {
            std::fs::remove_dir_all(&index_path)?;
            std::fs::create_dir_all(&index_path)?;
        }

        let new_index = SearchIndex::new(&index_path)?;
        let mut index = self.index.write().await;
        *index = new_index;

        Ok(())
    }
}