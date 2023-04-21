use actix_web::{
    get, post,
    web::{self, Json},
    HttpResponse,
};
use common::{context::Context, error};

use crate::service::{
    auth::{AuthService, Login, Token},
    user::CreateUser,
};

#[post("/api/auth/login")]
pub async fn login(context: Context, login: Json<Login>) -> error::Result<Json<Token>> {
    Ok(Json(AuthService::new(context).login(&login).await?))
}

#[post("/api/user")]
pub async fn create_user(
    context: Context,
    Json(user): web::Json<CreateUser>,
) -> error::Result<Json<()>> {
    Ok(Json(AuthService::new(context).authentication(user).await?))
}

#[get("/api/auth/verify/{code}")]
pub async fn verify_link(context: Context, code: web::Path<String>) -> error::Result<HttpResponse> {
    let result = AuthService::new(context)
        .verify_link(code.into_inner())
        .await?;

    if !result {
        return Ok(HttpResponse::NotFound().finish());
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
}
