use actix_web::{
    delete, get, patch, post,
    web::{Json, Path},
    HttpResponse,
};
use common::{
    context::GeneralContext,
    entities::user::{PublicUser, LinkedAccount},
    error,
    api::linked_accounts::AddLinkedAccount,
};
use serde_json::json;

use crate::service::user::{UserChange, UserService};

#[get("/api/user_by_email/{id}")]
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

#[get("/api/user/{id}")]
pub async fn find_user(context: GeneralContext, id: Path<String>) -> error::Result<HttpResponse> {
    let user = UserService::new(context).find(id.parse()?).await?;
    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/api/user/my_user")]
pub async fn my_user(context: GeneralContext) -> error::Result<HttpResponse> {
    let user = UserService::new(context).my_user().await?;
    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[post("/api/user/{id}/linked_account")]
pub async fn add_linked_account(
    context: GeneralContext,
    id: Path<String>,
    Json(data): Json<AddLinkedAccount>,
) -> error::Result<Json<LinkedAccount>> {
    Ok(Json(
        UserService::new(context).create_linked_account(id.parse()?, data).await?
    ))
}

// #[patch("/api/user/{user_id}/linked_account/{account_id}")]
// pub async fn patch_linked_account(
//     context: GeneralContext,
//     id: Path<(String, String)>,
// ) -> error::Result<Json<LinkedAccount>> {
//     Ok(Json(
//         UserService::new(context).delete_linked_account(id.0.parse()?, id.1.clone()).await?
//     ))
// }

#[delete("/api/user/{user_id}/linked_account/{account_id}")]
pub async fn delete_linked_account(
    context: GeneralContext,
    id: Path<(String, String)>,
) -> error::Result<Json<LinkedAccount>> {
    Ok(Json(
        UserService::new(context).delete_linked_account(id.0.parse()?, id.1.clone()).await?
    ))
}

#[patch("/api/user/{id}")]
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

#[delete("/api/user/{id}")]
pub async fn delete_user(
    context: GeneralContext,
    id: Path<String>,
) -> error::Result<Json<PublicUser>> {
    Ok(Json(UserService::new(context).delete(id.parse()?).await?))
}
