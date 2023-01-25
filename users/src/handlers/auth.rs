use std::result;

use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use common::auth_session::{jwt_from_header, AuthSession};
use common::ruleset::Ruleset;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    constants::MAX_DURATION,
    error::{Error, OuterError, Result},
    repositories::{
        token::{TokenModel, TokenRepository},
        user::UserRepository,
    },
    ruleset::Login,
    utils::jwt::{self, create},
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    token: String,
}

#[utoipa::path(
    request_body(
        content = LoginRequest
    ),
    responses(
        (status = 200, description = "Authorizes the user and return his token", body = LoginResponse)
    )
)]
#[post("/api/auth/login")]
pub async fn login(
    Json(data): web::Json<LoginRequest>,
    users_repo: web::Data<UserRepository>,
    tokens_repo: web::Data<TokenRepository>,
) -> Result<web::Json<LoginResponse>> {
    let Some(user) = users_repo.find_by_email(&data.email).await? else {
        return Err(Error::Outer(OuterError::UserNotFound));
    };

    if Login::request_access(&data, &user) {
        return Err(Error::Outer(OuterError::PasswordsDoesntMatch));
    }

    let token = TokenModel {
        created: Utc::now().naive_utc(),
        token: uuid::Uuid::new_v4().to_string(),
        user_id: user.id,
    };

    tokens_repo.create(&token).await?;

    let session = AuthSession {
        user_id: user.id.to_hex(),
        token: token.token,
    };

    let response = LoginResponse {
        token: jwt::create(session)?,
    };

    Ok(web::Json(response))
}

pub(super) async fn verify_token(
    req: &HttpRequest,
    repo: &web::Data<TokenRepository>,
) -> Result<result::Result<(TokenModel, AuthSession), String>> {
    let Some(jwt) = jwt_from_header(&req) else {
        return Ok(Err("Error: Failed to parse header".to_string()));
    };

    let Some(session) = jwt::verify(&jwt)? else {
        return Ok(Err("Error: Json web token has invalid signature".to_string()));
    };
    let Some(token) = repo.find(&session.token).await? else {
        return Ok(Err("Error: Invalid token".to_string()))
    };

    let token_duration = Utc::now().naive_utc() - token.created;

    if token_duration > *MAX_DURATION {
        return Ok(Err("Error: Token expired".to_string()));
    }

    return Ok(Ok((token, session)));
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RestoreResponse {
    token: String,
}

#[utoipa::path(
    security(
        ("BearerAuth" = []),
    ),
    responses(
        (status = 200, description = "New authorization token, but old became expired", body = RestoreResponse)
    )
)]
#[post("/api/auth/restore")]
pub async fn restore(
    req: HttpRequest,
    repo: web::Data<TokenRepository>,
) -> Result<web::Json<RestoreResponse>> {
    let Ok((mut token, session)) = verify_token(&req, &repo).await? else {
        return Err(Error::Outer(OuterError::UserNotFound));
    };

    repo.delete(&token.token).await?.unwrap();

    token.token = Uuid::new_v4().to_string();
    token.created = Utc::now().naive_utc();

    repo.create(&token).await?;

    let session = AuthSession {
        user_id: session.user_id,
        token: token.token,
    };

    let response = RestoreResponse {
        token: create(session)?,
    };

    Ok(web::Json(response))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Get authenticated user's data", body = Option<AuthSession>)
    )
)]
#[get("/api/auth/verify")]
pub async fn verify(req: HttpRequest, repo: web::Data<TokenRepository>) -> Result<HttpResponse> {
    let res = verify_token(&req, &repo)
        .await?
        .ok()
        .map(|(_, session)| session);

    Ok(HttpResponse::Ok().json(res))
}
