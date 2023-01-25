use serde::{Deserialize, Serialize};

pub mod jwt;
pub mod prelude;

#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    Customer,
    Auditor,
}
