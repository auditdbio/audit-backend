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
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "audit" => Ok(ParentEntitySource::Audit),
            "auditor" => Ok(ParentEntitySource::Auditor),
            "customer" => Ok(ParentEntitySource::Customer),
            "chat" => Ok(ParentEntitySource::Chat),
            "project" => Ok(ParentEntitySource::Project),
            "user" => Ok(ParentEntitySource::User),
            "organization" => Ok(ParentEntitySource::Organization),
            "other" => Ok(ParentEntitySource::Other),
            _ => Err(anyhow::anyhow!(
                "Parent entity source invalid value. Accepted values are: Audit, Auditor, Customer, Chat, Project, User, Organization, Other"
            ))
        }
    }
}

impl ParentEntitySource {
    pub fn to_string(&self) -> String {
        match self {
            ParentEntitySource::Audit => "Audit".to_string(),
            ParentEntitySource::Auditor => "Auditor".to_string(),
            ParentEntitySource::Customer => "Customer".to_string(),
            ParentEntitySource::Chat => "Chat".to_string(),
            ParentEntitySource::Project => "Project".to_string(),
            ParentEntitySource::User => "User".to_string(),
            ParentEntitySource::Organization => "Organization".to_string(),
            ParentEntitySource::Other => "Other".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FileEntity {
    Avatar,
    ChatAttachment,
    Report,
    Temporary,
    Other,
}

impl FromStr for FileEntity {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "avatar" => Ok(FileEntity::Avatar),
            "chatattachment" => Ok(FileEntity::ChatAttachment),
            "report" => Ok(FileEntity::Report),
            "temporary" => Ok(FileEntity::Temporary),
            "other" => Ok(FileEntity::Other),
            _ => Err(anyhow::anyhow!(
                "File entity invalid value. Accepted values are: Avatar, ChatAttachment, Report, Other"
            ))
        }
    }
}

impl FileEntity {
    pub fn to_string(&self) -> String {
        match self {
            FileEntity::Avatar => "Avatar".to_string(),
            FileEntity::ChatAttachment => "ChatAttachment".to_string(),
            FileEntity::Report => "Report".to_string(),
            FileEntity::Temporary => "Temporary".to_string(),
            FileEntity::Other => "Other".to_string(),
        }
    }
}
