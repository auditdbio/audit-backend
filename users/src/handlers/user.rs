use std::str::FromStr;

use actix_web::{
    delete, get, patch, post,
    web::{Json, Path},
};
use common::{context::Context, error};
use mongodb::bson::oid::ObjectId;

use crate::service::user::{CreateUser, PublicUser, UserChange, UserService};

#[post("/api/user")]
pub async fn create_user(
    context: Context,
    user: Json<CreateUser>,
) -> error::Result<Json<PublicUser>> {
    Ok(Json(
        UserService::new(context)
            .create_user(user.into_inner())
            .await?,
    ))
}

#[get("/api/user/{id}")]
pub async fn find_user(context: Context, id: Path<String>) -> error::Result<Json<PublicUser>> {
    Ok(Json(
        UserService::new(context)
            .find_user(ObjectId::from_str(&id)?)
            .await?,
    ))
}

#[patch("/api/user")]
pub async fn change_user(
    context: Context,
    user: Json<UserChange>,
) -> error::Result<Json<PublicUser>> {
    Ok(Json(
        UserService::new(context)
            .change_user(user.into_inner())
            .await?,
    ))
}

#[delete("/api/user/{id}")]
pub async fn delete_user(context: Context, id: Path<String>) -> error::Result<Json<PublicUser>> {
    Ok(Json(
        UserService::new(context)
            .delete_user(ObjectId::from_str(&id)?)
            .await?,
    ))
}
