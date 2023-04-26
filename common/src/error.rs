#[derive(Debug)]
pub struct ServiceError {
    err: anyhow::Error,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ServiceError: {}", self.err)
    }
}

impl actix_web::error::ResponseError for ServiceError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::BAD_REQUEST
    }
}

impl<E: Into<anyhow::Error>> From<E> for ServiceError {
    fn from(err: E) -> ServiceError {
        ServiceError { err: err.into() }
    }
}

pub type Result<T> = std::result::Result<T, ServiceError>;
