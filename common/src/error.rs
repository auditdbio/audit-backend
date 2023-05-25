#[derive(Debug)]
pub struct ServiceError {
    code: u16,
    err: anyhow::Error,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ServiceError: {}", self.err)
    }
}

impl actix_web::error::ResponseError for ServiceError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::from_u16(self.code).unwrap()
    }
}

impl<E: Into<anyhow::Error>> From<E> for ServiceError {
    fn from(err: E) -> ServiceError {
        ServiceError { code: 400, err: err.into() }
    }
}

pub trait AddCode {
    fn code(self, code: u16) -> ServiceError;
}


impl AddCode for anyhow::Error {
    fn code(self, code: u16) -> ServiceError {
        ServiceError { code, err: self }
    }
}


pub type Result<T> = std::result::Result<T, ServiceError>;
