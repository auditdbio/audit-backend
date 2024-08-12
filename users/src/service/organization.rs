use chrono::Utc;
use mongodb::bson::{Bson, doc, oid::ObjectId};
use serde::{Serialize, Deserialize};
use std::env::var;
use regex::Regex;

use common::{
    api::{
        auditor::request_auditor,
        customer::request_customer,
        events::{post_event, PublicEvent, EventPayload},
        linked_accounts::{
            create_github_account,
            create_linked_in_account,
            create_x_account,
            AddLinkedAccount,
            GetGithubAccessToken,
            LinkedService,
        },
        organization::{new_org_link_id, GetOrganizationQuery},
        user::get_by_id,
        send_notification,
        NewNotification,
    },
    context::GeneralContext,
    entities::{
        contacts::Contacts,
        organization::{
            Organization, OrganizationMember,
            OrgAccessLevel, PublicOrganization,
            MyOrganizations,
        },
        role::Role,
        user::{LinkedAccount, PublicLinkedAccount},
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
    pub link_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewOrganizationMember<Id> {
    pub user_id: Id,
    pub access_level: Vec<OrgAccessLevel>,
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
        let user_id = auth.id().ok_or(anyhow::anyhow!("No user id found"))?;

        if create_org.name.is_empty() {
            return Err(anyhow::anyhow!("Name is required").code(400));
        }

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let owner = OrganizationMember {
            user_id: user_id.to_hex(),
            access_level: vec![OrgAccessLevel::Owner, OrgAccessLevel::Representative, OrgAccessLevel::Editor]
        };

        let org_id = ObjectId::new();

        let link_id = new_org_link_id(
            &self.context,
            create_org.name.clone(),
            org_id.to_hex(),
            true,
        ).await?;

        let organization = Organization {
            id: org_id,
            owner: owner.clone(),
            name: create_org.name,
            link_id,
            contacts: create_org.contacts,
            avatar: create_org.avatar,
            linked_accounts: vec![],
            organization_type: create_org.organization_type,
            members: vec![owner],
            invites: vec![],
            created_at: Utc::now().timestamp_micros(),
            last_modified: Utc::now().timestamp_micros(),
        };

        organizations.insert(&organization).await?;

        Ok(PublicOrganization::new(&self.context, organization, true).await?)
    }

    pub async fn get_organization(
        &self,
        org_id: ObjectId,
        query: GetOrganizationQuery,
    ) -> error::Result<PublicOrganization> {
        let with_members = query.with_members.unwrap_or(true);
        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let Some(organization) = organizations
            .find("id", &Bson::ObjectId(org_id))
            .await? else {
            return Err(anyhow::anyhow!("Organization not found").code(404));
        };

        Ok(PublicOrganization::new(&self.context, organization, with_members).await?)
    }

    pub async fn get_organization_by_link_id(
        &self,
        link_id: String,
    ) -> error::Result<PublicOrganization> {
        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let Some(organization) = organizations
            .find("link_id", &Bson::String(link_id.clone().to_lowercase()))
            .await? else {
            return Err(anyhow::anyhow!("Organization not found").code(404));
        };

        Ok(PublicOrganization::new(&self.context, organization, true).await?)
    }

    pub async fn my_organizations(&self) -> error::Result<MyOrganizations> {
        let auth = self.context.auth();
        let id = auth.id().unwrap();

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let user = get_by_id(&self.context, auth, id.clone()).await?;
        let current_role = Role::parse(&user.current_role)?;

        let organizations_as_owner = organizations
            .find_many("owner.user_id", &Bson::String(id.to_hex()))
            .await?
            .into_iter()
            .filter(|org| org.organization_type == current_role)
            .collect::<Vec<Organization<ObjectId>>>();

        let mut as_owner = vec![];
        for org in organizations_as_owner {
            let public_org = PublicOrganization::new(&self.context, org.clone(), true).await?;
            as_owner.push(public_org);
        }

        let organizations_as_member = organizations
            .find_many("members", &Bson::Document(doc! {"$elemMatch": { "user_id": id.to_hex() }}))
            .await?
            .into_iter()
            .filter(|org| org.organization_type == current_role)
            .collect::<Vec<Organization<ObjectId>>>();

        let mut as_member = vec![];
        for org in organizations_as_member {
            let public_org = PublicOrganization::new(&self.context, org.clone(), true).await?;
            if public_org.owner.user_id != id.to_hex() {
                as_member.push(public_org);
            }
        }

        let organizations_invite = organizations
            .find_many("invites", &Bson::Document(doc! {"$elemMatch": { "user_id": id.to_hex() }}))
            .await?
            .into_iter()
            .filter(|org| org.organization_type == current_role)
            .collect::<Vec<Organization<ObjectId>>>();

        let mut invites = vec![];
        for org in organizations_invite {
            let public_org = PublicOrganization::new(&self.context, org.clone(), true).await?;
            if public_org.owner.user_id != id.to_hex() {
                invites.push(public_org);
            }
        }

        Ok(MyOrganizations {
            owner: as_owner,
            member: as_member,
            invites,
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

        let public_org = PublicOrganization::new(
            &self.context,
            organization.clone(),
            false
        ).await?;

        if organization.organization_type == Role::Auditor {
            for mut member in new_members {
                if organization
                    .members
                    .iter()
                    .find(|m| m.user_id == member.user_id.to_hex())
                    .is_some() {
                    continue
                }

                if organization
                    .invites
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

                let event = PublicEvent::new(
                    member.user_id,
                    EventPayload::OrganizationInvite(public_org.clone())
                );
                post_event(&self.context, event, self.context.server_auth()).await?;

                let mut new_notification: NewNotification =
                    serde_json::from_str(include_str!("../../templates/organization_invite.txt"))?;

                new_notification.user_id = Some(member.user_id);
                new_notification.role = organization.organization_type.stringify().to_string();
                let variables = vec![("organization".to_owned(), organization.name.clone())];
                new_notification
                    .links
                    .push(format!("/o/{}", organization.link_id));

                send_notification(&self.context, false, true, new_notification, variables).await?;

                organization.invites.push(OrganizationMember {
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

                if organization
                    .invites
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

                let event = PublicEvent::new(
                    member.user_id,
                    EventPayload::OrganizationInvite(public_org.clone())
                );
                post_event(&self.context, event, self.context.server_auth()).await?;

                let mut new_notification: NewNotification =
                    serde_json::from_str(include_str!("../../templates/organization_invite.txt"))?;

                new_notification.user_id = Some(member.user_id);
                new_notification.role = organization.organization_type.stringify().to_string();
                let variables = vec![("organization".to_owned(), organization.name.clone())];
                new_notification
                    .links
                    .push(format!("/o/{}", organization.link_id));

                send_notification(&self.context, false, true, new_notification, variables).await?;

                organization.invites.push(OrganizationMember {
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

    pub async fn confirm_invite(
        &self,
        org_id: ObjectId,
    ) -> error::Result<PublicOrganization> {
        let auth = self.context.auth();
        let user_id = auth.id().unwrap();

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let Some(mut organization) = organizations
            .find("id", &Bson::ObjectId(org_id))
            .await? else {
            return Err(anyhow::anyhow!("Organization not found").code(404));
        };

        let Some(member) = organization
            .invites
            .iter()
            .find(|m| m.user_id == user_id.to_hex())
            .cloned() else {
            return Err(anyhow::anyhow!("Invite not found").code(404));
        };

        organization.invites.retain(|inv| inv.user_id != user_id.to_hex());
        organization.members.push(member);

        organizations.delete("id", &organization.id).await?;
        organizations.insert(&organization).await?;

        Ok(PublicOrganization::new(&self.context, organization, true).await?)
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

    pub async fn cancel_invite(
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
            .invites
            .iter()
            .find(|member| member.user_id == user_id.to_hex())
            .cloned() else {
            return Err(anyhow::anyhow!("Member not found").code(404));
        };

        organization.invites.retain(|member| member.user_id != user_id.to_hex());

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
            if name.is_empty() {
                return Err(anyhow::anyhow!("Name is required").code(400));
            }
            organization.name = name;
        }

        if let Some(contacts) = change.contacts {
            if let Some(ref email) = contacts.email {
                let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
                if !email_regex.is_match(&email) && !email.is_empty() {
                    return Err(anyhow::anyhow!("Incorrect email").code(400));
                }
            }
            organization.contacts = contacts;
        }

        if change.avatar.is_some() {
            organization.avatar = change.avatar;
        }

        if let Some(link_id) = change.link_id {
            let regex = Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
            if !regex.is_match(&link_id) {
                return Err(
                    anyhow::anyhow!("Link ID may only contain alphanumeric characters, hyphens or underscore").code(400)
                );
            }

            let new_link_id = new_org_link_id(
                &self.context,
                link_id,
                organization.id.to_hex(),
                false,
            ).await?;

            organization.link_id = new_link_id;
        }

        organizations.delete("id", &organization.id).await?;
        organizations.insert(&organization).await?;

        Ok(PublicOrganization::new(&self.context, organization, true).await?)
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

        if organization.owner.user_id == user_id.to_hex() {
            return Err(anyhow::anyhow!("The owner has full access level by default.").code(400));
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

    pub async fn add_organization_linked_account(
        &self,
        org_id: ObjectId,
        data: AddLinkedAccount,
    ) -> error::Result<PublicLinkedAccount> {
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

        let linked_account: LinkedAccount;

        if data.service == LinkedService::GitHub {
            let github_auth = GetGithubAccessToken {
                code: data.clone().code,
                client_id: var("GITHUB_CLIENT_ID").unwrap(),
                client_secret: var("GITHUB_CLIENT_SECRET").unwrap(),
            };

            linked_account = create_github_account(github_auth).await?;
            if organization
                .linked_accounts
                .iter()
                .find(|acc| acc.id == linked_account.id)
                .is_some() {
                return Err(anyhow::anyhow!("Account has already been added").code(400))
            }

            organization.linked_accounts.push(linked_account.clone());
        } else if data.service == LinkedService::X {
            linked_account = create_x_account(data).await?;
            if organization
                .linked_accounts
                .iter()
                .find(|acc| acc.id == linked_account.id)
                .is_some() {
                return Err(anyhow::anyhow!("Account has already been added").code(400))
            }

            organization.linked_accounts.push(linked_account.clone());
        } else if data.service == LinkedService::LinkedIn {
            linked_account = create_linked_in_account(data).await?;
            if organization
                .linked_accounts
                .iter()
                .find(|acc| acc.id == linked_account.id)
                .is_some() {
                return Err(anyhow::anyhow!("Account has already been added").code(400))
            }

            organization.linked_accounts.push(linked_account.clone());
        } else {
            return Err(anyhow::anyhow!("Account connection error").code(400));
        }

        organizations.delete("id", &organization.id).await?;
        organizations.insert(&organization).await?;

        Ok(PublicLinkedAccount::from(linked_account))
    }

    pub async fn delete_organization_linked_account(
        &self,
        org_id: ObjectId,
        account_id: String,
    ) -> error::Result<PublicLinkedAccount> {
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

        let Some(linked_account) = organization.linked_accounts
            .iter()
            .find(|acc| acc.id == account_id)
            .cloned()
            else {
                return Err(anyhow::anyhow!("Linked account not found").code(404));
            };

        organization.linked_accounts.retain(|acc| acc.id != account_id);

        organizations.delete("id", &organization.id).await?;
        organizations.insert(&organization).await?;

        Ok(PublicLinkedAccount::from(linked_account))
    }

    pub async fn get_invites(
        &self,
        org_id: ObjectId,
    ) -> error::Result<Vec<OrganizationMember>> {
        let auth = self.context.auth();
        let current_id = auth.id().unwrap();

        let organizations = self.context.try_get_repository::<Organization<ObjectId>>()?;

        let Some(organization) = organizations
            .find("id", &Bson::ObjectId(org_id))
            .await? else {
            return Err(anyhow::anyhow!("Organization not found").code(404));
        };

        if current_id.to_hex() != organization.owner.user_id {
            return Err(anyhow::anyhow!("User is not available to read this organization").code(403));
        }

        Ok(organization.invites)
    }
}
