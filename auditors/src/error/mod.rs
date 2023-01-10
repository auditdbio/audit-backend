use actix_web::ResponseError;
use derive_more::{Error, Display};

#[derive(Debug, Error, Display)]
pub enum Error {
    MongoError(mongodb::error::Error),
}

impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Self::MongoError(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl ResponseError for Error {}
