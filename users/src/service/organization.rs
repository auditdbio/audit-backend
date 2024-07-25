use chrono::Utc;
use mongodb::bson::{Bson, doc, oid::ObjectId};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct NewOrganizationMember<Id> {
    pub user_id: Id,
    pub access_level: Vec<OrgAccessLevel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyOrganizations {
    pub owner: Vec<PublicOrganization>,
    pub member: Vec<PublicOrganization>,
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
    ) -> error::Result<PublicOrganization> {
        let auth = self.context.auth();
        let id = auth.id().ok_or(anyhow::anyhow!("No user id found"))?;

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let owner = OrganizationMember {
            user_id: id.to_hex(),
            access_level: vec![OrgAccessLevel::Owner, OrgAccessLevel::Representative, OrgAccessLevel::Editor]
        };

        let organization = Organization {
            id: ObjectId::new(),
            owner: owner.clone(),
            name: create_org.name,
            contacts: create_org.contacts,
            avatar: create_org.avatar,
            linked_accounts: vec![],
            organization_type: create_org.organization_type,
            members: create_org.members.unwrap_or(vec![owner]),
            last_modified: Utc::now().timestamp_micros(),
            created_at: Utc::now().timestamp_micros(),
        };

        organizations.insert(&organization).await?;

        Ok(PublicOrganization::new(&self.context, organization).await?)
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

        Ok(PublicOrganization::new(&self.context, organization).await?)
    }

    pub async fn my_organizations(&self) -> error::Result<MyOrganizations> {
        let auth = self.context.auth();
        let id = auth.id().unwrap();

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let organizations_as_owner = organizations
            .find_many("owner.user_id", &Bson::String(id.to_hex()))
            .await?;
        let mut as_owner = vec![];
        for org in organizations_as_owner {
            let public_org = PublicOrganization::new(&self.context, org.clone()).await?;
            as_owner.push(public_org);
        }

        let organizations_as_member = organizations
            .find_many("members", &Bson::Document(doc! {"$elemMatch": { "user_id": id.to_hex() }}))
            .await?;
        let mut as_member = vec![];
        for org in organizations_as_member {
            let public_org = PublicOrganization::new(&self.context, org.clone()).await?;
            if public_org.owner.user_id != id.to_hex() {
                as_member.push(public_org);
            }
        }

        Ok(MyOrganizations {
            owner: as_owner,
            member: as_member,
        })
    }

    pub async fn add_members(
        &self,
        org_id: ObjectId,
        new_members: Vec<NewOrganizationMember<ObjectId>>
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
            for mut member in new_members {
                if organization
                    .members
                    .iter()
                    .find(|m| m.user_id == member.user_id.to_hex())
                    .is_some() {
                    continue
                }

                let auditor = match request_auditor(&self.context, member.user_id, auth.clone()).await {
                    Ok(auditor) => auditor,
                    _ => continue
                };
                if auditor.is_empty() {
                    continue;
                }

                member.access_level.retain(|access| access != &OrgAccessLevel::Owner);

                organization.members.push(OrganizationMember {
                    user_id: member.user_id.to_hex(),
                    access_level: member.access_level,
                })
            }
        } else if organization.organization_type == Role::Customer {
            for mut member in new_members {
                if organization
                    .members
                    .iter()
                    .find(|m| m.user_id == member.user_id.to_hex())
                    .is_some() {
                    continue
                }

                let customer = match request_customer(&self.context, member.user_id, auth.clone()).await {
                    Ok(customer) => customer,
                    _ => continue
                };
                if customer.user_id.is_empty() {
                    continue;
                }

                member.access_level.retain(|access| access != &OrgAccessLevel::Owner);

                organization.members.push(OrganizationMember {
                    user_id: member.user_id.to_hex(),
                    access_level: member.access_level,
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

        if organization.owner.user_id == user_id.to_hex() {
            return Err(anyhow::anyhow!("Another owner must be assigned").code(400));
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

        Ok(PublicOrganization::new(&self.context, organization).await?)
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
            member.access_level = data.clone();
            member.clone()
        } else {
            return Err(anyhow::anyhow!("Member not found").code(404));
        };

        if data.contains(&OrgAccessLevel::Owner) {
            if let Some(old_owner) = organization
                .members
                .iter_mut()
                .find(|member| member.user_id == current_id.to_hex())
            {
                old_owner.access_level.retain(|lvl| lvl.clone() != OrgAccessLevel::Owner);
                organization.owner = member.clone();
            }
        }

        organizations.delete("id", &organization.id).await?;
        organizations.insert(&organization).await?;

        Ok(member)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicOrganizationMember {
    pub user_id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub access_level: Vec<OrgAccessLevel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicOrganization {
    pub id: String,
    pub owner: PublicOrganizationMember,
    pub name: String,
    pub contacts: Contacts,
    pub avatar: Option<String>,
    pub linked_accounts: Vec<PublicLinkedAccount>,
    pub organization_type: Role,
    pub members: Vec<PublicOrganizationMember>,
    pub last_modified: i64,
    pub created_at: i64,
}

impl PublicOrganization {
    pub async fn new(
        context: &GeneralContext,
        org: Organization<ObjectId>,
    ) -> error::Result<PublicOrganization> {
        let auth = context.auth();
        let current_id = auth.id().unwrap();

        let linked_accounts = org
            .linked_accounts
            .into_iter()
            .map(|acc| PublicLinkedAccount::from(acc))
            .filter(|acc| acc.is_public)
            .collect();

        let is_member = org.owner.user_id == current_id.to_hex() || org
            .members
            .iter()
            .find(|m| m.user_id == current_id.to_hex())
            .is_some();

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

        let owner = Self::get_public_member(
            &context,
            org.organization_type,
            org.owner
        ).await?;

        Ok(PublicOrganization {
            id: org.id.to_hex(),
            owner,
            name: org.name,
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
