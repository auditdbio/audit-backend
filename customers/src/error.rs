use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use common::inner_error::InnerError;
use derive_more::{Display, Error};
use mongodb::error;

#[derive(Debug, Display, Error)]
pub enum Error {
    Inner(InnerError),
    Outer(OuterError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Display, Error)]
pub enum OuterError {
    #[display(fmt = "Project not found")]
    ProjectNotFound,
}

impl From<error::Error> for Error {
    fn from(err: error::Error) -> Self {
        Error::Inner(InnerError::MongoError(err))
    }
}

impl From<InnerError> for Error {
    fn from(value: InnerError) -> Self {
        Error::Inner(value)
    }
}

impl From<OuterError> for Error {
    fn from(value: OuterError) -> Self {
        Error::Outer(value)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Error::Inner(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Outer(_) => StatusCode::BAD_REQUEST,
        }
    }
}
