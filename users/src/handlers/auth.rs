use actix_web::{post, web::Json};
use common::{context::Context, error};

use crate::service::auth::{AuthService, Login, Token};

#[post("/api/auth/login")]
pub async fn login(context: Context, login: Json<Login>) -> error::Result<Json<Token>> {
    Ok(Json(AuthService::new(context).login(&login).await?))
}
