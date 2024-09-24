use serde::{Serialize, Deserialize};
use std::str::FromStr;
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ParentEntity {
    pub id: ObjectId,
    pub source: ParentEntitySource,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ParentEntitySource {
    Audit,
    Auditor,
    Customer,
    Chat,
    Project,
    User,
    Organization,
    Other,
}

impl FromStr for ParentEntitySource {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "audit" => Ok(ParentEntitySource::Audit),
            "auditor" => Ok(ParentEntitySource::Auditor),
            "customer" => Ok(ParentEntitySource::Customer),
            "chat" => Ok(ParentEntitySource::Chat),
            "project" => Ok(ParentEntitySource::Project),
            "user" => Ok(ParentEntitySource::User),
            "organization" => Ok(ParentEntitySource::Organization),
            _ => Ok(ParentEntitySource::Other),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FileEntity {
    Avatar,
    ChatAttachment,
    Report,
    Other,
}

impl FromStr for FileEntity {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "avatar" => Ok(FileEntity::Avatar),
            "chatattachment" => Ok(FileEntity::ChatAttachment),
            "report" => Ok(FileEntity::Report),
            _ => Ok(FileEntity::Other),
        }
    }
}
