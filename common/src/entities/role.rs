use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::error::{self, AddCode};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum Role {
    #[serde(alias = "customer")]
    Customer,
    #[serde(alias = "auditor")]
    Auditor,
}

impl Role {
    pub fn parse(s: &str) -> error::Result<Role> {
        match s.to_lowercase().as_str() {
            "customer" => Ok(Role::Customer),
            "auditor" => Ok(Role::Auditor),
            _ => Err(anyhow::anyhow!("Invalid role: {}", s).code(400)),
        }
    }

    pub fn stringify(&self) -> &'static str {
        match self {
            Role::Customer => "customer",
            Role::Auditor => "auditor",
        }
    }
}
