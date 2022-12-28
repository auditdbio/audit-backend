use serde::{Serialize, Deserialize};

pub mod jwt;
pub mod prelude;

#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    Customer,
    Auditor,
}

#[macro_export]
macro_rules! internal_error {
    ($expr:expr) => {
        match $expr {
            Ok(ok) => ok,
            Err(err) => return actix_web::HttpResponse::InternalServerError().body(println!("{:?}", err)),
        }
    }
}