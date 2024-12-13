use mongodb::bson::{Document, oid::ObjectId, to_document};
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
    repository::{Entity, HasLastModified},
    services::{API_PREFIX, PROTOCOL, USERS_SERVICE},
    impl_has_last_modified,
};
use crate::auth::Auth;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrgAccessLevel {
    Representative,
    Editor,
    Owner,
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

impl_has_last_modified!(Organization<ObjectId>);

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
    pub fn stringify(self) -> Organization<String> {
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

impl From<Organization<ObjectId>> for Option<Document> {
    fn from(organization: Organization<ObjectId>) -> Self {
        let organization = organization.stringify();
        let mut document = to_document(&organization).unwrap();
        if !organization.contacts.public_contacts {
            document.remove("contacts");
        }

        document.insert(
            "request_url",
            format!(
                "{}://{}/{}/organization/data",
                PROTOCOL.as_str(),
                USERS_SERVICE.as_str(),
                API_PREFIX.as_str(),
            ),
        );

        document.remove("last_modified");
        document.remove("linked_accounts");
        document.remove("members");
        document.remove("invites");
        document.insert("kind", "organization");
        Some(document)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicOrganizationMember {
    pub user_id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub access_level: OrgAccessLevel,
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
        match auth {
            Auth::None => log::info!("User is not authorized"),
            _ => {
                if let Some(current_id) = auth.id() {
                    is_member = org.owner.user_id == current_id.to_hex() || org
                        .members
                        .iter()
                        .any(|m| m.user_id == current_id.to_hex());
                }
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
        let auth = context.server_auth();

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
            access_level: member.access_level.into_iter().max().unwrap_or(OrgAccessLevel::Representative),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyOrganizations {
    pub owner: Vec<PublicOrganization>,
    pub member: Vec<PublicOrganization>,
    pub invites: Vec<PublicOrganization>,
}
