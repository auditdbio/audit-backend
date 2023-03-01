use std::result;

use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use chrono::{Duration, Utc};
use common::ruleset::Ruleset;
use common::{
    auth_session::{jwt_from_header, AuthSession},
    entities::user::User,
};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::{Error, OuterError, Result},
    repositories::{
        token::{TokenModel, TokenRepo},
        user::UserRepo,
    },
    ruleset::Login,
    utils::jwt::{self, create},
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
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
    users_repo: web::Data<UserRepo>,
    tokens_repo: web::Data<TokenRepo>,
) -> Result<web::Json<LoginResponse>> {
    let Some(user) = users_repo.find_by_email(&data.email).await? else {
        return Err(Error::Outer(OuterError::UserNotFound));
    };

    if !Login::request_access(&data, &user) {
        return Err(Error::Outer(OuterError::PasswordsDoesntMatch));
    }
    let res = Utc::now() + Duration::days(1);
    let token = TokenModel {
        exp: res.naive_utc().timestamp() as usize,
        token: uuid::Uuid::new_v4().to_string(),
        user_id: user.id,
    };

    tokens_repo.create(&token).await?;

    let session = AuthSession {
        user_id: user.id,
        token: token.token,
        exp: token.exp,
    };

    let response = LoginResponse {
        token: jwt::create(session)?,
        user,
    };

    Ok(web::Json(response))
}

pub(super) async fn verify_token(
    req: &HttpRequest,
    repo: &web::Data<TokenRepo>,
) -> Result<result::Result<(TokenModel, AuthSession), String>> {
    let Some(jwt) = jwt_from_header(&req) else {
        return Ok(Err("Error: Failed to parse header".to_string()));
    };

    let Some(session) = jwt::verify(&jwt)? else {
        return Ok(Err("Error: Json web token has invalid signature".to_string()));
    };
    let Some(token) = repo.find_by_token(session.token.clone()).await? else {
        return Ok(Err("Error: Invalid token".to_string()))
    };

    return Ok(Ok((token, session)));
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RestoreResponse {
    token: String,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, description = "New authorization token, but old became expired", body = RestoreResponse)
    )
)]
#[post("/api/auth/restore")]
pub async fn restore(
    req: HttpRequest,
    repo: web::Data<TokenRepo>,
) -> Result<web::Json<RestoreResponse>> {
    let Ok((mut token, session)) = verify_token(&req, &repo).await? else {
        return Err(Error::Outer(OuterError::UserNotFound));
    };

    repo.delete(token.token).await?.unwrap();

    token.token = Uuid::new_v4().to_string();
    token.exp = (Utc::now() + Duration::days(1)).naive_utc().timestamp() as usize;

    repo.create(&token).await?;

    let session = AuthSession {
        user_id: session.user_id,
        token: token.token,
        exp: token.exp,
    };

    let response = RestoreResponse {
        token: create(session)?,
    };

    Ok(web::Json(response))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, description = "Get authenticated user's data", body = Option<AuthSession>)
    )
)]
#[get("/api/auth/verify")]
pub async fn verify(req: HttpRequest, repo: web::Data<TokenRepo>) -> Result<HttpResponse> {
    let res = verify_token(&req, &repo)
        .await?
        .ok()
        .map(|(_, session)| session);

    Ok(HttpResponse::Ok().json(res))
}

#[cfg(test)]
mod tests {
    use actix_web::test::{self, init_service};
    use common::auth_session::AuthSession;
    use mongodb::bson::oid::ObjectId;

    use crate::{create_test_app, PostUserRequest};

    #[actix_web::test]
    async fn test_login() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            exp: 100000000,
        };

        let mut app = init_service(create_test_app(test_user)).await;

        let req = test::TestRequest::post()
            .uri("/api/users")
            .set_json(&PostUserRequest {
                name: "test".to_string(),
                email: "test@gmail.com".to_string(),
                password: "test".to_string(),
                current_role: "Customer".to_string(),
            })
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&super::LoginRequest {
                email: "test@gmail.com".to_string(),
                password: "test".to_string(),
            })
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}
