use actix_web::{
    delete, get, patch,
    web::{Json, Path},
    HttpResponse,
};
use common::{context::GeneralContext, entities::user::PublicUser, error};
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

#[get("/user/my_user")]
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
