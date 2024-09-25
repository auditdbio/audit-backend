use serde::{Serialize, Deserialize};
use std::str::FromStr;
use mongodb::bson::oid::ObjectId;

use crate::{
    impl_has_last_modified,
    repository::{Entity, HasLastModified}
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    pub id: ObjectId,
    pub last_modified: i64,
    pub path: String,
    pub extension: String,
    pub private: bool,
    pub allowed_users: Vec<ObjectId>,
    pub author: Option<ObjectId>,
    pub access_code: Option<String>,
    pub original_name: Option<String>,
    pub parent_entity: Option<ParentEntity<ObjectId>>,
    pub file_entity: Option<FileEntity>,
    #[serde(default)]
    pub is_rewritable: bool,
}

impl_has_last_modified!(Metadata);

impl Entity for Metadata {
    fn id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ParentEntity<Id> {
    pub id: Id,
    pub source: ParentEntitySource,
}

impl ParentEntity<ObjectId> {
    pub fn stringify(self) -> ParentEntity<String> {
        ParentEntity {
            id: self.id.to_hex(),
            source: self.source,
        }
    }
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
