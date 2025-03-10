use std::path::Path;
use sled::Db;

use crate::error::ServiceResult;

const LAST_SYNC_KEY: &[u8] = b"last_sync_timestamp";

pub struct Storage {
    db: Db,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> ServiceResult<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn get_last_sync_timestamp(&self) -> ServiceResult<Option<i64>> {
        Ok(self
            .db
            .get(LAST_SYNC_KEY)?
            .map(|ivec| i64::from_be_bytes(ivec.as_ref().try_into().unwrap())))
    }

    pub fn set_last_sync_timestamp(&self, timestamp: i64) -> ServiceResult<()> {
        self.db.insert(LAST_SYNC_KEY, &timestamp.to_be_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    pub fn clear(&self) -> ServiceResult<()> {
        self.db.clear()?;
        self.db.flush()?;
        Ok(())
    }
} 