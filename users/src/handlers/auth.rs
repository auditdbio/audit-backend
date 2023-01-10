use std::result;

use actix_web::{HttpResponse, HttpRequest, post, get, web::{self, Json}};
use chrono::{Utc, Duration};
use common::{AuthSession, jwt_from_header};
use mongodb::bson::doc;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use uuid::Uuid;
use lazy_static::lazy_static;
use common::ruleset::Ruleset;

use crate::{repositories::{user::UserRepository, token::{TokenRepository, TokenModel}}, utils::jwt::{self, create}, error::Result, ruleset::Login};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    email: String,
    pub password: String,
}

lazy_static! {
    static ref MAX_DURATION: Duration = Duration::days(1);
}

#[utoipa::path(
    request_body(
        content = LoginRequest
    ),
    responses(
        (status = 200, description = "Authorizes the user and return his token", body = String)
    )
)]
#[post("/api/auth/login")]
pub async fn login(
    Json(data): web::Json<LoginRequest>,
    users_repo: web::Data<UserRepository>,
    tokens_repo: web::Data<TokenRepository>,
) -> Result<HttpResponse> {
    let Some(user) = users_repo.find_by_email(&data.email).await? else {
        return Ok(HttpResponse::BadRequest().body("User not found"));
    };

    if Login::request_access(&data, &user ) {
        return Ok(HttpResponse::BadRequest().body("Passwords doesn't match"));
    }

    let token = TokenModel {
        created: Utc::now().naive_utc(),
        token: uuid::Uuid::new_v4().to_string(),
        user_id: user.id,
    };

   tokens_repo.create(&token).await?;

    let session = AuthSession {
        email: user.email,
        user_id: user.id.to_hex(),
        token: token.token,
    };

    let jwt = jwt::create(session)?;

    Ok(HttpResponse::Ok().body(jwt))
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
        return Ok(Err("Error: Token expired".to_string()))
    }

    return Ok(Ok((token, session)))
}


#[utoipa::path(
    security(
        ("BearerAuth" = []),
    ),
    responses(
        (status = 200, description = "New authorization token, but old became expired", body = String)
    )
)]
#[post("/api/auth/restore")]
pub async fn restore(
    req: HttpRequest,
    repo: web::Data<TokenRepository>,
) -> Result<HttpResponse> {
    let (mut token, session) = match verify_token(&req, &repo).await? {
        Ok(res) => res,
        Err(s) => return Ok(HttpResponse::Unauthorized().body(s)),
    };

    repo.delete(&token.token).await?.unwrap();

    token.token = Uuid::new_v4().to_string();
    token.created = Utc::now().naive_utc();

    repo.create(&token).await?;

    let session = AuthSession {
        email: session.email,
        user_id: session.user_id,
        token: token.token,
    };

    let jwt = create(session)?;

    Ok(HttpResponse::Ok().json(doc!{ "token": jwt }))
}


#[utoipa::path(
    responses(
        (status = 200, description = "Get authenticated user's data", body = Option<AuthSession>)
    )
)]
#[get("/api/auth/verify")]
pub async fn verify(
    req: HttpRequest,
    repo: web::Data<TokenRepository>,
) -> Result<HttpResponse> {
    let res = verify_token(&req, &repo)
        .await?
        .ok()
        .map(|(_, session)| session);

    Ok(HttpResponse::Ok().json(res))
}
