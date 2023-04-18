use actix_web::{
    delete, get, patch, post,
    web::{Json, Path},
    HttpResponse,
};
use common::{context::Context, error};
use serde_json::json;

use crate::service::user::{CreateUser, PublicUser, UserChange, UserService};

#[post("/api/user")]
pub async fn create_user(
    context: Context,
    user: Json<CreateUser>,
) -> error::Result<Json<PublicUser>> {
    Ok(Json(
        UserService::new(context).create(user.into_inner()).await?,
    ))
}

#[get("/api/user/{id}")]
pub async fn find_user(context: Context, id: Path<String>) -> error::Result<HttpResponse> {
    let user = UserService::new(context).find(id.parse()?).await?;
    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/api/my_user")]
pub async fn my_user(context: Context) -> error::Result<HttpResponse> {
    let user = UserService::new(context).my_user().await?;
    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[patch("/api/user/{id}")]
pub async fn change_user(
    context: Context,
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
pub async fn delete_user(context: Context, id: Path<String>) -> error::Result<Json<PublicUser>> {
    Ok(Json(UserService::new(context).delete(id.parse()?).await?))
}
