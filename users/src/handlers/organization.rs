use actix_web::{
    delete, get, post, patch,
    web::{Json, Path},
};
use mongodb::bson::oid::ObjectId;

use common::{
    context::GeneralContext,
    entities::organization::{Organization, OrganizationMember, OrgAccessLevel},
    error,
};

use crate::service::organization::{
    ChangeOrganization, CreateOrganization,
    MyOrganizations, NewOrganizationMember,
    OrganizationService, PublicOrganization
};

#[post("/organization")]
pub async fn create_organization(
    context: GeneralContext,
    Json(data): Json<CreateOrganization>,
) -> error::Result<Json<Organization<String>>> {
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
) -> error::Result<Json<PublicOrganization>> {
    Ok(Json(
        OrganizationService::new(context).get_organization(org_id.parse()?).await?
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
