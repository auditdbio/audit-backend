use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::{
    entities::{
        contacts::Contacts,
        user::LinkedAccount,
    },
    repository::Entity,
};
use crate::entities::role::Role;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrgAccessLevel {
    Owner,
    Representative,
    Editor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrganizationMember {
    pub user_id: String,
    pub access_level: Vec<OrgAccessLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Organization<Id> {
    pub id: Id,
    pub owner: OrganizationMember,
    pub name: String,
    pub contacts: Contacts,
    pub avatar: Option<String>,
    pub linked_accounts: Vec<LinkedAccount>,
    pub organization_type: Role,
    pub members: Vec<OrganizationMember>,
    pub last_modified: i64,
    pub created_at: i64,
}

impl Organization<String> {
    pub fn parse(self) -> Organization<ObjectId> {
        Organization {
            id: self.id.parse().unwrap(),
            owner: self.owner,
            name: self.name,
            contacts: self.contacts,
            avatar: self.avatar,
            linked_accounts: self.linked_accounts,
            organization_type: self.organization_type,
            members: self.members,
            last_modified: self.last_modified,
            created_at: self.created_at,
        }
    }
}

impl Organization<ObjectId> {
    pub fn parse(self) -> Organization<String> {
        Organization {
            id: self.id.to_hex(),
            owner: self.owner,
            name: self.name,
            contacts: self.contacts,
            avatar: self.avatar,
            linked_accounts: self.linked_accounts,
            organization_type: self.organization_type,
            members: self.members,
            last_modified: self.last_modified,
            created_at: self.created_at,
        }
    }
}

impl Entity for Organization<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}
