use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("MongoDB error: {0}")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Tantivy error: {0}")]
    Tantivy(#[from] tantivy::TantivyError),

    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid query: {0}")]
    Query(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::Query(msg) => {
                HttpResponse::BadRequest().json(json!({ "error": msg }))
            }
            _ => HttpResponse::InternalServerError()
                .json(json!({ "error": "Internal server error" })),
        }
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>; 