use std::{
    fs::File,
    io::{self, Read, Write},
};

use actix_web::web::{self, Bytes};
use mongodb::bson::oid::ObjectId;

pub struct FilesRepository {}

impl FilesRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create(&self, bytes: Bytes) -> ObjectId {
        let file_id = ObjectId::new();
        web::block(move || {
            let mut file = File::create(format!("./files/{}", file_id)).unwrap();
            file.write_all(&bytes).unwrap();
        })
        .await
        .unwrap();
        file_id
    }

    pub async fn get(&self, id: &ObjectId) -> Bytes {
        let id = id.clone();
        web::block(move || {
            let file = File::open(format!("./files/{}", id)).unwrap();
            let bytes = Bytes::from(
                file.bytes()
                    .collect::<Result<Vec<u8>, io::Error>>()
                    .unwrap(),
            );
            bytes
        })
        .await
        .unwrap()
    }
}
