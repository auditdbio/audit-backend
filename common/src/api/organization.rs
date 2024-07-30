use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    context::GeneralContext,
    error,
    entities::{
        organization::{PublicOrganization, OrgAccessLevel},
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

    Ok(context
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
        .await?
        .json::<PublicOrganization>()
        .await?
    )
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
