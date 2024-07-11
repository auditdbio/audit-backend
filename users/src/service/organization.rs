use chrono::Utc;
use mongodb::bson::{Bson, oid::ObjectId};
use serde::{Serialize, Deserialize};

use common::{
    api::{auditor::request_auditor, customer::request_customer},
    context::GeneralContext,
    entities::{
        contacts::Contacts,
        organization::{
            Organization, OrganizationMember,
            OrgAccessLevel,
        },
        role::Role,
        user::PublicLinkedAccount,
    },
    error::{self, AddCode},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrganization {
    pub name: String,
    pub contacts: Contacts,
    pub organization_type: Role,
    pub avatar: Option<String>,
    pub members: Option<Vec<OrganizationMember>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeOrganization {
    pub name: Option<String>,
    pub contacts: Option<Contacts>,
    pub avatar: Option<String>,
}

pub struct OrganizationService {
    pub context: GeneralContext,
}

impl OrganizationService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn create_organization(
        &self,
        create_org: CreateOrganization
    ) -> error::Result<Organization<String>> {
        let auth = self.context.auth();
        let id = auth.id().ok_or(anyhow::anyhow!("No user id found"))?;

        let (username, user_avatar) = if create_org.organization_type == Role::Auditor {
            let auditor = request_auditor(&self.context, id, auth.clone()).await?;
            if auditor.is_empty() {
                return Err(anyhow::anyhow!("Auditor not found").code(404))
            }
            (auditor.first_name().clone() + " " + auditor.last_name(), auditor.avatar().to_string())
        } else if create_org.organization_type == Role::Customer {
            let customer = request_customer(&self.context, id, auth.clone()).await?;
            if customer.user_id.is_empty() {
                return Err(anyhow::anyhow!("Customer not found").code(404))
            }
            (customer.first_name + " " + &customer.last_name, customer.avatar)
        } else {
            return Err(anyhow::anyhow!("Unknown organization type").code(400));
        };

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let organization = Organization {
            id: ObjectId::new(),
            owner: OrganizationMember {
                user_id: id.to_hex(),
                username,
                avatar: Some(user_avatar),
                access_level: vec![OrgAccessLevel::Owner]
            },
            name: create_org.name,
            contacts: create_org.contacts,
            avatar: create_org.avatar,
            linked_accounts: vec![],
            organization_type: create_org.organization_type,
            members: create_org.members.unwrap_or(vec![]),
            last_modified: Utc::now().timestamp_micros(),
            created_at: Utc::now().timestamp_micros(),
        };

        organizations.insert(&organization).await?;

        Ok(organization.parse())
    }

    pub async fn get_organization(
        &self,
        org_id: ObjectId,
    ) -> error::Result<PublicOrganization> {
        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let Some(organization) = organizations
            .find("id", &Bson::ObjectId(org_id))
            .await? else {
            return Err(anyhow::anyhow!("Organization not found").code(404));
        };

        Ok(PublicOrganization::from(organization))
    }

    pub async fn add_members(
        &self,
        org_id: ObjectId,
        new_members: Vec<ObjectId>
    ) -> error::Result<Vec<OrganizationMember>> {
        let auth = self.context.auth();
        let id = auth.id().unwrap();

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let Some(mut organization) = organizations
            .find("id", &Bson::ObjectId(org_id))
            .await? else {
            return Err(anyhow::anyhow!("Organization not found").code(404));
        };

        if id.to_hex() != organization.owner.user_id {
            return Err(anyhow::anyhow!("User is not available to change this organization").code(403));
        }

        if organization.organization_type == Role::Auditor {
            for member_id in new_members {
                let auditor = match request_auditor(&self.context, member_id, auth.clone()).await {
                    Ok(auditor) => auditor,
                    _ => continue
                };
                if auditor.is_empty() {
                    continue;
                }

                organization.members.push(OrganizationMember {
                    user_id: member_id.to_hex(),
                    username: format!("{} {}", auditor.first_name(), auditor.last_name()),
                    avatar: Some(auditor.avatar().clone()),
                    access_level: vec![],
                })
            }
        } else if organization.organization_type == Role::Customer {
            for member_id in new_members {
                let customer = match request_customer(&self.context, member_id, auth.clone()).await {
                    Ok(customer) => customer,
                    _ => continue
                };
                if customer.user_id.is_empty() {
                    continue;
                }

                organization.members.push(OrganizationMember {
                    user_id: member_id.to_hex(),
                    username: format!("{} {}", customer.first_name, customer.last_name),
                    avatar: Some(customer.avatar),
                    access_level: vec![],
                })
            }
        } else {
            return Err(anyhow::anyhow!("Unknown organization type").code(400));
        }

        organizations.delete("id", &organization.id).await?;
        organizations.insert(&organization).await?;

        Ok(organization.members)
    }

    pub async fn delete_member(
        &self,
        org_id: ObjectId,
        user_id: ObjectId,
    ) -> error::Result<OrganizationMember> {
        let auth = self.context.auth();
        let current_id = auth.id().unwrap();

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let Some(mut organization) = organizations
            .find("id", &Bson::ObjectId(org_id))
            .await? else {
            return Err(anyhow::anyhow!("Organization not found").code(404));
        };

        if current_id.to_hex() != organization.owner.user_id && current_id != user_id {
            return Err(anyhow::anyhow!("User is not available to change this organization").code(403));
        }

        let Some(member) = organization
            .members
            .iter()
            .find(|member| member.user_id == user_id.to_hex())
            .cloned()
        else {
            return Err(anyhow::anyhow!("Member not found").code(404));
        };

        organization.members.retain(|member| member.user_id != user_id.to_hex());

        organizations.delete("id", &organization.id).await?;
        organizations.insert(&organization).await?;

        Ok(member)
    }

    pub async fn change_organization(
        &self,
        org_id: ObjectId,
        change: ChangeOrganization,
    ) -> error::Result<PublicOrganization> {
        let auth = self.context.auth();
        let current_id = auth.id().unwrap();

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let Some(mut organization) = organizations
            .find("id", &Bson::ObjectId(org_id))
            .await? else {
            return Err(anyhow::anyhow!("Organization not found").code(404));
        };

        if current_id.to_hex() != organization.owner.user_id {
            return Err(anyhow::anyhow!("User is not available to change this organization").code(403));
        }

        if let Some(name) = change.name {
            organization.name = name;
        }

        if let Some(contacts) = change.contacts {
            organization.contacts = contacts;
        }

        if change.avatar.is_some() {
            organization.avatar = change.avatar;
        }

        organizations.delete("id", &organization.id).await?;
        organizations.insert(&organization).await?;

        Ok(PublicOrganization::from(organization))
    }

    pub async fn change_access(
        &self,
        org_id: ObjectId,
        user_id: ObjectId,
        data: Vec<OrgAccessLevel>,
    ) -> error::Result<OrganizationMember> {
        let auth = self.context.auth();
        let current_id = auth.id().unwrap();

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let Some(mut organization) = organizations
            .find("id", &Bson::ObjectId(org_id))
            .await? else {
            return Err(anyhow::anyhow!("Organization not found").code(404));
        };

        if current_id.to_hex() != organization.owner.user_id {
            return Err(anyhow::anyhow!("User is not available to change this organization").code(403));
        }

        let member = if let Some(member) = organization
            .members
            .iter_mut()
            .find(|member| member.user_id == user_id.to_hex())
        {
            member.access_level = data;
            member.clone()
        } else {
            return Err(anyhow::anyhow!("Member not found").code(404));
        };

        organizations.delete("id", &organization.id).await?;
        organizations.insert(&organization).await?;

        Ok(member)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicOrganization {
    pub id: String,
    pub owner: OrganizationMember,
    pub name: String,
    pub contacts: Contacts,
    pub avatar: Option<String>,
    pub linked_accounts: Vec<PublicLinkedAccount>,
    pub organization_type: Role,
    pub members: Vec<OrganizationMember>,
    pub last_modified: i64,
    pub created_at: i64,
}

impl From<Organization<ObjectId>> for PublicOrganization {
    fn from(org: Organization<ObjectId>) -> Self {
        let linked_accounts = org
            .linked_accounts
            .into_iter()
            .map(|acc| PublicLinkedAccount::from(acc))
            .filter(|acc| acc.is_public)
            .collect();

        let contacts = if org.contacts.public_contacts {
            org.contacts
        } else {
            Contacts {
                email: None,
                telegram: None,
                public_contacts: false,
            }
        };

        Self {
            id: org.id.to_hex(),
            owner: org.owner,
            name: org.name,
            contacts,
            avatar: org.avatar,
            linked_accounts,
            organization_type: org.organization_type,
            members: org.members,
            last_modified: org.last_modified,
            created_at: org.created_at,
        }
    }
}
