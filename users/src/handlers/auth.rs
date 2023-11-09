use std::env::var;
use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use mongodb::bson::{oid::ObjectId};
use common::{
    api::user::{CreateUser, GithubAuth, GetGithubAccessToken},
    context::GeneralContext,
    entities::user::User,
    auth::Auth,
    error::{self, AddCode},
};

use crate::service::{
    auth::{AuthService, ChangePasswordData, Login, Token, TokenResponce, create_auth_token},
    user::UserService,
};

#[post("/api/auth/login")]
pub async fn login(context: GeneralContext, login: Json<Login>) -> error::Result<Json<Token>> {
    Ok(Json(AuthService::new(context).login(&login).await?))
}

#[post("/api/auth/github")]
pub async fn github_auth(
    context: GeneralContext,
    Json(data): Json<GithubAuth>,
) -> error::Result<Json<Token>> {
    let github_auth = GetGithubAccessToken {
        code: data.code,
        client_id: var("GITHUB_CLIENT_ID").unwrap(),
        client_secret: var("GITHUB_CLIENT_SECRET").unwrap(),
    };

    let auth_service = AuthService::new(context.clone());
    let user_service = UserService::new(context);

    let github_user = auth_service
        .github_auth(github_auth, data.current_role)
        .await?;

    let existing_user = user_service
        .find_by_email(github_user.email.clone())
        .await?;

    let auth_result = match existing_user {
        Some(user) => create_auth_token(&user),
        None => {
            let verify_email = false;
            auth_service.authentication(github_user.clone(), verify_email).await?;
            let user = user_service
                .find_by_email(github_user.email.clone())
                .await?;

            match user {
                Some(user) => create_auth_token(&user),
                None => Err(anyhow::anyhow!("No user found").code(404)),
            }
        }
    };

    auth_result
}

#[post("/api/user")]
pub async fn create_user(
    context: GeneralContext,
    Json(user): web::Json<CreateUser>,
) -> error::Result<Json<User<String>>> {
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
            .await?,
    ))
}

#[get("/api/auth/verify/{code}")]
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

#[get("/api/auth/forgot_password/{email}")]
pub async fn forgot_password(
    context: GeneralContext,
    email: web::Path<String>,
) -> error::Result<HttpResponse> {
    AuthService::new(context)
        .forgot_password(email.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/api/auth/reset_password")]
pub async fn reset_password(
    context: GeneralContext,
    Json(code): web::Json<ChangePasswordData>,
) -> error::Result<HttpResponse> {
    AuthService::new(context).reset_password(code).await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/api/auth/restore_token")]
pub async fn restore_token(
    context: GeneralContext,
    req: HttpRequest,
) -> error::Result<Json<TokenResponce>> {
    Ok(Json(AuthService::new(context).restore(req).await?))
}
