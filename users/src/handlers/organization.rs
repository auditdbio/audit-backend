use actix_web::{
    delete, get, post, patch,
    web::{Json, Path, Query},
};
use mongodb::bson::oid::ObjectId;

use common::{
    api::{
        linked_accounts::AddLinkedAccount,
        organization::GetOrganizationQuery
    },
    context::GeneralContext,
    entities::{
        organization::{OrganizationMember, OrgAccessLevel, PublicOrganization},
        user::PublicLinkedAccount,
    },
    error,
};

use crate::service::organization::{
    ChangeOrganization, CreateOrganization,
    MyOrganizations, NewOrganizationMember,
    OrganizationService,
};

#[post("/organization")]
pub async fn create_organization(
    context: GeneralContext,
    Json(data): Json<CreateOrganization>,
) -> error::Result<Json<PublicOrganization>> {
    Ok(Json(
        OrganizationService::new(context).create_organization(data).await?
    ))
}

#[get("/my_organizations")]
pub async fn get_my_organizations(
    context: GeneralContext
) -> error::Result<Json<MyOrganizations>> {
    Ok(Json(
        OrganizationService::new(context).my_organizations().await?
    ))
}

#[get("/organization/{org_id}")]
pub async fn get_organization(
    context: GeneralContext,
    org_id: Path<String>,
    query: Query<GetOrganizationQuery>
) -> error::Result<Json<PublicOrganization>> {
    Ok(Json(
        OrganizationService::new(context)
            .get_organization(org_id.parse()?, query.into_inner())
            .await?
    ))
}

#[post("/organization/{org_id}/members")]
pub async fn add_members(
    context: GeneralContext,
    org_id: Path<String>,
    Json(data): Json<Vec<NewOrganizationMember<ObjectId>>>,
) -> error::Result<Json<Vec<OrganizationMember>>> {
    Ok(Json(
        OrganizationService::new(context).add_members(org_id.parse()?, data).await?
    ))
}

#[delete("/organization/{org_id}/members/{user_id}")]
pub async fn delete_member(
    context: GeneralContext,
    path: Path<(String, String)>,
) -> error::Result<Json<OrganizationMember>> {
    let (org_id, user_id) = path.into_inner();
    Ok(Json(
        OrganizationService::new(context).delete_member(org_id.parse()?, user_id.parse()?).await?
    ))
}

#[patch("/organization/{org_id}")]
pub async fn change_organization(
    context: GeneralContext,
    org_id: Path<String>,
    Json(data): Json<ChangeOrganization>,
) -> error::Result<Json<PublicOrganization>> {
    Ok(Json(
        OrganizationService::new(context)
            .change_organization(org_id.parse()?, data)
            .await?
    ))
}

#[patch("/organization/{org_id}/members/{user_id}")]
pub async fn change_access(
    context: GeneralContext,
    path: Path<(String, String)>,
    Json(data): Json<Vec<OrgAccessLevel>>,
) -> error::Result<Json<OrganizationMember>> {
    let (org_id, user_id) = path.into_inner();
    Ok(Json(
        OrganizationService::new(context)
            .change_access(org_id.parse()?, user_id.parse()?, data)
            .await?
    ))
}

#[post("/organization/{org_id}/linked_account")]
pub async fn add_organization_linked_account(
    context: GeneralContext,
    path: Path<String>,
    Json(data): Json<AddLinkedAccount>,
) -> error::Result<Json<PublicLinkedAccount>> {
    let org_id = path.into_inner();
    Ok(Json(
        OrganizationService::new(context)
            .add_organization_linked_account(org_id.parse()?, data)
            .await?
    ))
}

#[delete("/organization/{org_id}/linked_account/{acc_id}")]
pub async fn delete_organization_linked_account(
    context: GeneralContext,
    path: Path<(String, String)>,
) -> error::Result<Json<PublicLinkedAccount>> {
    let (org_id, acc_id) = path.into_inner();
    Ok(Json(
        OrganizationService::new(context)
            .delete_organization_linked_account(org_id.parse()?, acc_id)
            .await?
    ))
}
