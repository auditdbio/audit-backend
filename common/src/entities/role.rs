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
            Role::Customer => "Customer",
            Role::Auditor => "Auditor",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum ChatRole {
    #[serde(alias = "customer")]
    Customer,
    #[serde(alias = "auditor")]
    Auditor,
    #[serde(alias = "organization")]
    Organization,
}

impl From<Role> for ChatRole {
    fn from(role: Role) -> Self {
        match role {
            Role::Auditor => ChatRole::Auditor,
            Role::Customer => ChatRole::Customer,
        }
    }
}

impl ChatRole {
    pub fn stringify(&self) -> &'static str {
        match self {
            ChatRole::Customer => "Customer",
            ChatRole::Auditor => "Auditor",
            ChatRole::Organization => "Organization",
        }
    }
}
