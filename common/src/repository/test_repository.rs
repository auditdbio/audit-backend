use std::sync::Mutex;

use async_trait::async_trait;
use mongodb::bson::{self, oid::ObjectId, Bson, Document};
use serde::{de::DeserializeOwned, Serialize};

use crate::error;

use super::{Entity, Repository};

pub struct TestRepository<T> {
    _t: std::marker::PhantomData<T>,
    pub db: Mutex<Vec<Bson>>,
}

impl<T> Default for TestRepository<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TestRepository<T> {
    pub fn new() -> Self {
        Self {
            _t: std::marker::PhantomData,
            db: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl<T> Repository<T> for TestRepository<T>
where
    T: Entity + Clone + PartialEq + Send + Sync + Serialize + DeserializeOwned,
{
    async fn insert(&self, item: &T) -> error::Result<bool> {
        let mut db = self.db.lock().unwrap();

        let contains = db
            .iter()
            .any(|x| x.as_document().unwrap().get_object_id("id").unwrap() == item.id());
        if !contains {
            db.push(bson::to_bson(&item).unwrap());
        }
        Ok(!contains)
    }

    async fn find(&self, field: &str, value: &Bson) -> error::Result<Option<T>> {
        let db = self.db.lock().unwrap();
        Ok(db
            .iter()
            .find(|x| x.as_document().unwrap().get(field).unwrap() == value)
            .cloned()
            .map(|x| bson::from_bson(x).unwrap()))
    }

    async fn delete(&self, field: &str, id: &ObjectId) -> error::Result<Option<T>> {
        let mut db = self.db.lock().unwrap();
        let result = db
            .iter()
            .find(|x| &x.as_document().unwrap().get_object_id(field).unwrap() == id)
            .cloned()
            .map(|x| bson::from_bson(x).unwrap());

        let pos = db
            .iter()
            .position(|x| &x.as_document().unwrap().get_object_id(field).unwrap() == id);

        pos.map(|x| db.remove(x));

        Ok(result)
    }

    async fn find_all(&self, skip: u32, limit: u32) -> error::Result<Vec<T>> {
        let db = self.db.lock().unwrap();
        Ok(db
            .iter()
            .skip(skip as usize)
            .take(limit as usize)
            .map(|x| bson::from_bson(x.clone()).unwrap())
            .collect())
    }

    async fn find_many(&self, field: &str, value: &Bson) -> error::Result<Vec<T>> {
        let db = self.db.lock().unwrap();
        Ok(db
            .iter()
            .filter(|x| x.as_document().unwrap().get(field).unwrap() == value)
            .map(|x| bson::from_bson(x.clone()).unwrap())
            .collect())
    }

    async fn find_many_limit(
        &self,
        field: &str,
        value: &Bson,
        skip: i32,
        limit: i32,
        sort: Option<Document>,
    ) -> error::Result<(Vec<T>, u64)> {
        let db = self.db.lock().unwrap();
        let result: Vec<_> = db
            .iter()
            .filter(|x| x.as_document().unwrap().get(field).unwrap() == value)
            .map(|x| bson::from_bson(x.clone()).unwrap())
            .collect();
        let skip = skip as usize;
        let limit = limit as usize;

        Ok((result[skip..=limit].to_vec(), db.len() as u64))
    }

    async fn get_all_since(&self, since: i64) -> error::Result<Vec<T>> {
        let db = self.db.lock().unwrap();
        Ok(db
            .iter()
            .filter(|x| x.as_document().unwrap().get_i64("last_modified").unwrap() > since)
            .map(|x| bson::from_bson(x.clone()).unwrap())
            .collect())
    }

    async fn find_all_by_ids(&self, id: &str, ids: Vec<ObjectId>) -> error::Result<Vec<T>> {
        let db = self.db.lock().unwrap();
        Ok(db
            .iter()
            .filter(|x| ids.contains(&x.as_document().unwrap().get_object_id(id).unwrap()))
            .map(|x| bson::from_bson(x.clone()).unwrap())
            .collect())
    }
}

// #[async_trait]
// impl<T> TaggableEntityRepository<T> for TestRepository<T>
// where
//     T: Entity + TaggableEntity + Clone + PartialEq + Send + Sync + Serialize + DeserializeOwned,
// {
//     async fn find_by_tags(
//         &self,
//         tags: Vec<String>,
//         skip: u32,
//         limit: u32,
//     ) -> Result<Vec<T>, Self::Error> {
//         let db = self.db.lock().unwrap();
//         Ok(db
//             .iter()
//             .filter_map(|elem| {
//                 let elem = bson::from_bson::<T>(elem.clone()).unwrap();
//                 for tag in &tags {
//                     if elem.tags().contains(&tag) {
//                         return Some(elem);
//                     }
//                 }
//                 None
//             })
//             .skip(skip as usize)
//             .take(limit as usize)
//             .collect())
//     }
// }
