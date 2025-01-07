use sled::Db;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::task;

const LAST_MODIFIED_KEY: &[u8] = b"last_modified";

#[async_trait]
pub trait Vars {
    async fn get_last_modified(&self) -> i64;
    async fn set_last_modified(&self, value: i64);
}

#[derive(Clone)]
pub struct SledVars {
    db: Arc<Db>,
}

impl SledVars {
    pub async fn new(db: Arc<Db>) -> Self {
        let db_clone = Arc::clone(&db);
        // Initialize last_modified with 0 if it doesn't exist
        task::spawn_blocking(move || {
            if db_clone.get(LAST_MODIFIED_KEY).unwrap().is_none() {
                db_clone.insert(LAST_MODIFIED_KEY, &0i64.to_be_bytes()).unwrap();
            }
        }).await.unwrap();
        
        Self { db }
    }
}

#[async_trait]
impl Vars for SledVars {
    async fn get_last_modified(&self) -> i64 {
        let db = Arc::clone(&self.db);
        task::spawn_blocking(move || {
            let bytes = db.get(LAST_MODIFIED_KEY).unwrap().unwrap();
            i64::from_be_bytes(bytes.as_ref().try_into().unwrap())
        }).await.unwrap()
    }

    async fn set_last_modified(&self, value: i64) {
        let db = Arc::clone(&self.db);
        task::spawn_blocking(move || {
            db.insert(LAST_MODIFIED_KEY, &value.to_be_bytes()).unwrap();
        }).await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_vars() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(sled::open(temp_dir.path()).unwrap());
        let vars = SledVars::new(db).await;

        // Test initial value
        assert_eq!(vars.get_last_modified().await, 0);

        // Test setting and getting value
        vars.set_last_modified(42).await;
        assert_eq!(vars.get_last_modified().await, 42);
    }
} 