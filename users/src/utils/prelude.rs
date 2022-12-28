use jsonwebtoken::errors::ErrorKind;

pub trait IsSignatureError {
    type Res;
    fn is_signature(self) -> <Self as IsSignatureError>::Res;
}

impl<T> IsSignatureError for Result<T, jsonwebtoken::errors::Error> {
    type Res = crate::error::Result<Option<T>>;

    fn is_signature(self) -> <Self as IsSignatureError>::Res {
        match self {
            Ok(ok) => Ok(Some(ok)),
            Err(err) if err.kind() == &ErrorKind::InvalidSignature => Ok(None),
            Err(err) => Err(crate::error::Error::Jwt(err)),
        }
    }
}
