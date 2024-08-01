use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::{
    api::{auditor::request_auditor, customer::request_customer},
    context::GeneralContext,
    error::{self, AddCode},
    entities::{
        contacts::Contacts,
        role::Role,
        user::{LinkedAccount, PublicLinkedAccount},
    },
    repository::Entity,
};
use crate::auth::Auth;

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
    pub link_id: String,
    pub contacts: Contacts,
    pub avatar: Option<String>,
    pub linked_accounts: Vec<LinkedAccount>,
    pub organization_type: Role,
    pub last_modified: i64,
    pub created_at: i64,
    pub members: Vec<OrganizationMember>,
    #[serde(default)]
    pub invites: Vec<OrganizationMember>,
}

impl Organization<String> {
    pub fn parse(self) -> Organization<ObjectId> {
        Organization {
            id: self.id.parse().unwrap(),
            owner: self.owner,
            name: self.name,
            link_id: self.link_id,
            contacts: self.contacts,
            avatar: self.avatar,
            linked_accounts: self.linked_accounts,
            organization_type: self.organization_type,
            members: self.members,
            invites: self.invites,
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
            link_id: self.link_id,
            contacts: self.contacts,
            avatar: self.avatar,
            linked_accounts: self.linked_accounts,
            organization_type: self.organization_type,
            members: self.members,
            invites: self.invites,
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


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicOrganizationMember {
    pub user_id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub access_level: Vec<OrgAccessLevel>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicOrganization {
    pub id: String,
    pub owner: PublicOrganizationMember,
    pub name: String,
    pub link_id: String,
    pub contacts: Contacts,
    pub avatar: Option<String>,
    pub linked_accounts: Vec<PublicLinkedAccount>,
    pub organization_type: Role,
    pub members: Option<Vec<PublicOrganizationMember>>,
    pub last_modified: i64,
    pub created_at: i64,
}

impl PublicOrganization {
    pub async fn new(
        context: &GeneralContext,
        org: Organization<ObjectId>,
        with_members: bool,
    ) -> error::Result<PublicOrganization> {
        let auth = context.auth();

        let mut is_member = false;
        if let Auth::None = auth {
        } else {
            if let Some(current_id) = auth.id() {
                is_member = org.owner.user_id == current_id.to_hex() || org
                    .members
                    .iter()
                    .find(|m| m.user_id == current_id.to_hex())
                    .is_some();
            }
        }

        let linked_accounts = org
            .linked_accounts
            .into_iter()
            .map(|acc| PublicLinkedAccount::from(acc))
            .filter(|acc| acc.is_public)
            .collect();

        let contacts = if org.contacts.public_contacts || is_member {
            org.contacts.clone()
        } else {
            Contacts {
                email: None,
                telegram: None,
                public_contacts: false,
            }
        };

        let mut members = vec![];
        if with_members {
            for member in org.members {
                let public_member = match Self::get_public_member(
                    &context,
                    org.organization_type,
                    member
                ).await {
                    Ok(member) => member,
                    _ => continue,
                };
                members.push(public_member);
            }
        }

        let members = if with_members {
            Some(members)
        } else {
            None
        };

        let owner = Self::get_public_member(
            &context,
            org.organization_type,
            org.owner
        ).await?;

        Ok(PublicOrganization {
            id: org.id.to_hex(),
            owner,
            name: org.name,
            link_id: org.link_id,
            contacts,
            avatar: org.avatar,
            linked_accounts,
            organization_type: org.organization_type,
            members,
            last_modified: org.last_modified,
            created_at: org.created_at,
        })
    }

    async fn get_public_member(
        context: &GeneralContext,
        org_type: Role,
        member: OrganizationMember,
    ) -> error::Result<PublicOrganizationMember> {
        let auth = context.auth();

        let (username, avatar) = if org_type == Role::Auditor {
            let auditor = request_auditor(&context, member.user_id.parse()?, auth.clone()).await?;
            if auditor.is_empty() {
                return Err(anyhow::anyhow!("Auditor not found").code(404))
            }
            (auditor.first_name().clone() + " " + auditor.last_name(), auditor.avatar().to_string())
        } else if org_type == Role::Customer {
            let customer = request_customer(&context, member.user_id.parse()?, auth.clone()).await?;
            if customer.user_id.is_empty() {
                return Err(anyhow::anyhow!("Customer not found").code(404))
            }
            (customer.first_name + " " + &customer.last_name, customer.avatar)
        } else {
            return Err(anyhow::anyhow!("Unknown organization type").code(400));
        };

        Ok(PublicOrganizationMember {
            user_id: member.user_id,
            username,
            avatar: Some(avatar),
            access_level: member.access_level,
        })
    }
}
