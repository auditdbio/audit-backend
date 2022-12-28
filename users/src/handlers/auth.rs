use std::result;

use actix_web::{HttpResponse, HttpRequest, post, get, web::{self, Json}};
use chrono::{NaiveDateTime, Utc, Duration};
use mongodb::bson::{DateTime, doc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{repositories::{user::UserRepository, token::{TokenRepository, TokenModel}}, internal_error, utils::jwt::{Claims, self, create}, error::Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

lazy_static! {
    static ref MAX_DURATION: Duration = Duration::days(1);
}


#[post("/api/auth/login")]
pub async fn login(
    req: HttpRequest,
    Json(data): web::Json<LoginRequest>,
    users_repo: web::Data<UserRepository>,
    tokens_repo: web::Data<TokenRepository>,
) -> HttpResponse {
    let Some(user) = internal_error!(users_repo.find_by_email(&data.email).await) else {
        return HttpResponse::BadRequest().finish(); // TODO: add error description to body
    };

    if user.password != data.password {
        return HttpResponse::BadRequest().finish(); // TODO: add error description to body
    }

    let token = TokenModel {
        created: Utc::now().naive_utc(),
        token: uuid::Uuid::new_v4().to_string(),
        user_id: user.id.unwrap(),
    };

    let token = internal_error!(tokens_repo.create(&token).await);

    let claims = Claims {
        email: user.email,
        user_id: user.id.unwrap().to_hex(),
        token: token.token,
    };

    let jwt = internal_error!(jwt::create(claims));

    HttpResponse::Ok().json(doc!{ "token": jwt, "user": doc!{} })
}

fn jwt_from_header(req: &HttpRequest) -> Option<String> { // possibly make readable error
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|x| x.to_str().ok())
        .and_then(|x| x.strip_prefix("Bearer ")) // remove prefix
        .map(str::to_string);

    token
}

pub(super) async fn verify_token(
    req: &HttpRequest,
    repo: &web::Data<TokenRepository>,
) -> Result<result::Result<(TokenModel, Claims), String>> {
    let Some(jwt) = jwt_from_header(&req) else {
        return Ok(Err("Error: Failed to parse header".to_string()));
    };

    let Some(claims) = jwt::verify(&jwt)? else {
        return Ok(Err("Error: Json web token has invalid signature".to_string()));
    };

    let Some(token) = repo.find(&claims.token).await? else {
        return Ok(Err("Error: Invalid token".to_string()))
    };

    let token_duration = Utc::now().naive_utc() - token.created;

    if token_duration > *MAX_DURATION {
        return Ok(Err("Error: Token expired".to_string()))
    }

    return Ok(Ok((token, claims)))
}



#[post("/api/auth/restore")]
pub async fn restore(
    req: HttpRequest,
    repo: web::Data<TokenRepository>,
) -> HttpResponse {
    let (mut token, claims) = match internal_error!(verify_token(&req, &repo).await) {
        Ok(res) => res,
        Err(s) => return HttpResponse::Unauthorized().body(s),
    };

    internal_error!(repo.remove(&token.token).await).unwrap();

    token.token = Uuid::new_v4().to_string();

    internal_error!(repo.create(&token).await);

    let claims = Claims {
        email: claims.email,
        user_id: claims.user_id,
        token: token.token,
    };

    let jwt = internal_error!(create(claims));

    HttpResponse::Ok().json(doc!{ "token": jwt, "user": doc!{} })
}

#[get("/api/auth/verify")]
pub async fn verify(
    req: HttpRequest,
    repo: web::Data<TokenRepository>,
) -> HttpResponse {
    let res = internal_error!(verify_token(&req, &repo).await).is_ok();

    HttpResponse::Ok().json(res)
}
