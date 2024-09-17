use mongodb::bson::oid::ObjectId;
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    context::GeneralContext,
    error::{self, AddCode},
    entities::{
        organization::{PublicOrganization, OrgAccessLevel, PublicOrganizationMember, MyOrganizations},
    },
    services::{API_PREFIX, USERS_SERVICE, PROTOCOL},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetOrganizationQuery {
    pub with_members: Option<bool>,
}

pub async fn get_organization(
    context: &GeneralContext,
    id: ObjectId,
    query: Option<GetOrganizationQuery>,
) -> error::Result<PublicOrganization> {
    let with_members = if query.is_none() {
        true
    } else {
        query.unwrap().with_members.unwrap_or(true)
    };

    let response = context
        .make_request::<PublicOrganization>()
        .auth(context.auth())
        .get(format!(
            "{}://{}/{}/organization/{}?with_members={}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            id,
            with_members,
        ))
        .send()
        .await?;

       if response.status().is_success() {
           let organization: PublicOrganization = response.json().await?;
           Ok(organization)
       } else {
           Err(anyhow::anyhow!("Organization not found").code(404))
       }
}

pub async fn get_my_organizations(
    context: &GeneralContext,
) -> error::Result<MyOrganizations> {
    let organizations = context
        .make_request::<MyOrganizations>()
        .auth(context.auth())
        .get(format!(
            "{}://{}/{}/my_organizations",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
        ))
        .send()
        .await?
        .json::<MyOrganizations>()
        .await?;

    Ok(organizations)
}

pub async fn check_is_organization_user(
    context: &GeneralContext,
    org_id: ObjectId,
    access: Option<OrgAccessLevel>,
) -> error::Result<bool> {
    let auth = context.auth();
    let user_id = auth.id().unwrap();

    let org = get_organization(context, org_id, None).await?;
    let is_owner = org.owner.user_id == user_id.to_hex();

    let is_member = org.members
        .as_ref()
        .map_or(false, |members| {
            members
                .iter()
                .any(|m| {
                    m.user_id == user_id.to_hex() && match &access {
                        Some(access) => m.access_level.contains(access),
                        None => true,
                    }
                })
        });

    if !is_owner && !is_member {
        return Ok(false);
    }

    Ok(true)
}

pub async fn check_editor_rights(members: Vec<PublicOrganizationMember>, user_id: ObjectId) -> error::Result<()> {
    if members
        .iter()
        .find(|m| {
            m.user_id == user_id.to_hex()
                && (m.access_level.contains(&OrgAccessLevel::Editor)
                || m.access_level.contains(&OrgAccessLevel::Owner))
        })
        .is_none() {
        return Err(
            anyhow::anyhow!("The user doesn't have permission to create an audit").code(403)
        );
    }

    Ok(())
}

pub async fn new_org_link_id(
    context: &GeneralContext,
    link_id: String,
    org_id: String,
    add_postfix: bool,
) -> error::Result<String> {
    let regex = Regex::new(r"[^A-Za-z0-9_-]").unwrap();
    let link_id = regex.replace_all(&link_id, "").to_string().to_lowercase();

    let organization = context
        .make_request::<PublicOrganization>()
        .get(format!(
            "{}://{}/{}/organization/link_id/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            link_id,
        ))
        .auth(context.server_auth())
        .send()
        .await?;

    let is_taken = if organization.status().is_success() {
        organization.json::<PublicOrganization>().await.map_or_else(
            |_| false,
            |org| org.id != org_id
        )
    } else { false };

    if !add_postfix && is_taken.clone() {
        return Err(anyhow::anyhow!("This link id is already taken").code(400));
    }

    if add_postfix && is_taken {
        let rnd: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        let result_link_id = format!(
            "{}-{}{}",
            link_id,
            org_id.chars().rev().take(3).collect::<String>(),
            rnd,
        );
        return Ok(result_link_id.to_lowercase());
    }

    Ok(link_id.to_lowercase())
}
