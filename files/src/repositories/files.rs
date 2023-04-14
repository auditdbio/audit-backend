use std::{
    fs::File,
    io::{self, Read, Write},
};

use actix_web::web::Bytes;

#[derive(Clone, Copy)]
pub struct FilesRepository {}

impl FilesRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create(&self, bytes: Vec<u8>, path: String) {
        let mut file = File::create(format!("/auditdb-files/{}", path)).unwrap();
        file.write_all(&bytes).unwrap();
    }

    pub async fn get(&self, path: String) -> Bytes {
        let file = File::open(format!("/auditdb-files/{}", path)).unwrap();
        let bytes = Bytes::from(
            file.bytes()
                .collect::<Result<Vec<u8>, io::Error>>()
                .unwrap(),
        );
        bytes
    }

    pub async fn delete(&self, path: String) {
        std::fs::remove_file(format!("/auditdb-files/{}", path)).unwrap();
    }
}
