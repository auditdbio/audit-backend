use std::{
    fs::File,
    io::{self, Read, Write},
};

use actix_web::web::{self, Bytes};

#[derive(Clone, Copy)]
pub struct FilesRepository {}

impl FilesRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create(&self, bytes: Bytes, path: String) {
        web::block(move || {
            let mut file = File::create(format!("/auditdb-files/{}", path)).unwrap();
            file.write_all(&bytes).unwrap();
        })
        .await
        .unwrap();
    }

    pub async fn get(&self, path: String) -> Bytes {
        web::block(move || {
            let file = File::open(format!("./files/{}", path)).unwrap();
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

    pub async fn delete(&self, path: String) {
        web::block(move || {
            std::fs::remove_file(format!("./files/{}", path)).unwrap();
        })
        .await
        .unwrap()
    }
}
