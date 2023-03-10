use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum Role {
    #[serde(alias = "customer")]
    Customer,
    #[serde(alias = "auditor")]
    Auditor,
}
