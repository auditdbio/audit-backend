use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use common::{
    api::user::{CreateUser, GithubAuth},
    context::GeneralContext,
    entities::user::{LinkedAccount, User},
    error,
};
use serde::{Deserialize, Serialize};

use crate::service::auth::{AuthService, ChangePasswordData, Login, Token, TokenResponce};

#[post("/auth/login")]
pub async fn login(context: GeneralContext, login: Json<Login>) -> error::Result<Json<Token>> {
    Ok(Json(AuthService::new(context).login(&login).await?))
}

#[post("/auth/github")]
pub async fn github_auth(
    context: GeneralContext,
    Json(data): Json<GithubAuth>,
) -> error::Result<Json<Token>> {
    Ok(Json(AuthService::new(context).github_auth(data).await?))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateUserResponce {
    id: String,
    name: String,
    current_role: String,
    email: String,
    is_new: bool,
    linked_accounts: Option<Vec<LinkedAccount>>,
    is_passwordless: Option<bool>,
    link_id: String,
}

impl From<User<String>> for CreateUserResponce {
    fn from(user: User<String>) -> Self {
        Self {
            id: user.id,
            name: user.name,
            current_role: user.current_role,
            email: user.email,
            is_new: user.is_new,
            linked_accounts: user.linked_accounts,
            is_passwordless: user.is_passwordless,
            link_id: user.link_id,
        }
    }
}

#[post("/user")]
pub async fn create_user(
    context: GeneralContext,
    Json(user): web::Json<CreateUser>,
) -> error::Result<Json<CreateUserResponce>> {
    #[allow(unused_mut)]
    let mut use_email = true;

    #[cfg(feature = "test_server")]
    if user.use_email == Some(false) {
        use_email = false;
        log::info!("this registration is not using email verification")
    }

    Ok(Json(
        AuthService::new(context)
            .authentication(user, use_email)
            .await?
            .into(),
    ))
}

#[get("/auth/verify/{code}")]
pub async fn verify_link(
    context: GeneralContext,
    code: web::Path<String>,
) -> error::Result<HttpResponse> {
    let service = AuthService::new(context);
    let result = service.verify_link(code.into_inner()).await?;

    if !result {
        return Ok(HttpResponse::NotFound().finish());
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", "/sign-in"))
        .finish())
}

#[get("/auth/forgot_password/{email}")]
pub async fn forgot_password(
    context: GeneralContext,
    email: web::Path<String>,
) -> error::Result<HttpResponse> {
    AuthService::new(context)
        .forgot_password(email.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/auth/reset_password")]
pub async fn reset_password(
    context: GeneralContext,
    Json(code): web::Json<ChangePasswordData>,
) -> error::Result<HttpResponse> {
    AuthService::new(context).reset_password(code).await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/auth/restore_token")]
pub async fn restore_token(
    context: GeneralContext,
    req: HttpRequest,
) -> error::Result<Json<TokenResponce>> {
    Ok(Json(AuthService::new(context).restore(req).await?))
}
