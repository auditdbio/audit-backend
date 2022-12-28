use mongodb::error;


#[derive(Debug)]
pub enum Error {
    MongoError(error::Error),
    Jwt(jsonwebtoken::errors::Error),
    InvalidSignature,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<error::Error> for Error {
    fn from(err: error::Error) -> Self {
        Error::MongoError(err)
    }
}
