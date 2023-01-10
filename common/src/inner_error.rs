
use derive_more::{Display, Error};
#[derive(Debug, Display, Error)]
pub enum InnerError {
    MongoError(mongodb::error::Error),
    Jwt(jsonwebtoken::errors::Error),
    InvalidSignature,
}
