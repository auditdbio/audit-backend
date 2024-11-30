use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum Role {
    #[serde(alias = "customer")]
    Customer,
    #[serde(alias = "auditor")]
    Auditor,
}

impl Role {
    pub fn parse(s: &str) -> Result<Role, String> {
        match s {
            "customer" => Ok(Role::Customer),
            "auditor" => Ok(Role::Auditor),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }

    pub fn stringify(&self) -> &'static str {
        match self {
            Role::Customer => "Customer",
            Role::Auditor => "Auditor",
        }
    }
}
