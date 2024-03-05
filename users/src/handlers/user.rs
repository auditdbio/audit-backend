use actix_web::{
    delete, get, patch, post,
    web::{Json, Path, Query},
    HttpResponse,
};
use common::{
    context::GeneralContext,
    entities::user::{PublicUser, PublicLinkedAccount},
    error,
    api::linked_accounts::{AddLinkedAccount, UpdateLinkedAccount, AddWallet},
};
use serde_json::json;

use crate::service::user::{UserChange, UserService};

#[get("/user_by_email/{id}")]
pub async fn find_user_by_email(
    context: GeneralContext,
    id: Path<String>,
) -> error::Result<HttpResponse> {
    let user = UserService::new(context).find_by_email(id.parse()?).await?;
    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/user/{id}")]
pub async fn find_user(context: GeneralContext, id: Path<String>) -> error::Result<HttpResponse> {
    let user = UserService::new(context).find(id.parse()?).await?;
    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/my_user")]
pub async fn my_user(context: GeneralContext) -> error::Result<HttpResponse> {
    let user = UserService::new(context).my_user().await?;
    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[patch("/user/{id}")]
pub async fn change_user(
    context: GeneralContext,
    id: Path<String>,
    user: Json<UserChange>,
) -> error::Result<Json<PublicUser>> {
    Ok(Json(
        UserService::new(context)
            .change(id.parse()?, user.into_inner())
            .await?,
    ))
}

#[delete("/user/{id}")]
pub async fn delete_user(
    context: GeneralContext,
    id: Path<String>,
) -> error::Result<Json<PublicUser>> {
    Ok(Json(UserService::new(context).delete(id.parse()?).await?))
}

#[post("/user/{id}/linked_account")]
pub async fn add_linked_account(
    context: GeneralContext,
    id: Path<String>,
    Json(data): Json<AddLinkedAccount>,
) -> error::Result<Json<PublicLinkedAccount>> {
    Ok(Json(
        UserService::new(context).create_linked_account(id.parse()?, data).await?
    ))
}

#[patch("/user/{id}/linked_account/{account_id}")]
pub async fn patch_linked_account(
    context: GeneralContext,
    id: Path<(String, String)>,
    Json(data): Json<UpdateLinkedAccount>,
) -> error::Result<Json<PublicLinkedAccount>> {
    Ok(Json(
        UserService::new(context)
            .change_linked_account(id.0.parse()?, id.1.clone(), data)
            .await?
    ))
}

#[delete("/user/{id}/linked_account/{account_id}")]
pub async fn delete_linked_account(
    context: GeneralContext,
    id: Path<(String, String)>,
) -> error::Result<Json<PublicLinkedAccount>> {
    Ok(Json(
        UserService::new(context)
            .delete_linked_account(id.0.parse()?, id.1.clone())
            .await?
    ))
}

#[post("/user/{id}/wallet")]
pub async fn add_wallet(
    context: GeneralContext,
    id: Path<String>,
    Json(data): Json<AddWallet>,
) -> error::Result<Json<PublicLinkedAccount>> {
    Ok(Json(
        UserService::new(context).add_wallet(id.parse()?, data).await?
    ))
}

#[get("/github/{path:.*}")]
pub async fn proxy_github_api(
    context: GeneralContext,
    path: Path<String>,
    query: Query<Vec<(String, String)>>,
) -> error::Result<HttpResponse> {
    Ok(UserService::new(context).proxy_github_api(path.into_inner(), query.into_inner()).await?)
}
