use actix_web::{
    get, post,
    web::{Json, Path},
};
use common::{context::Context, error};

use crate::service::auth::{AuthService, Login, Token};

#[post("/api/auth/login")]
pub async fn login(context: Context, login: Json<Login>) -> error::Result<Json<Token>> {
    Ok(Json(AuthService::new(context).login(&login).await?))
}

#[get("/api/auth/send_code/{email}")]
pub async fn send_code(context: Context, email: Path<String>) -> error::Result<Json<()>> {
    Ok(Json(
        AuthService::new(context)
            .send_code(email.into_inner())
            .await?,
    ))
}
