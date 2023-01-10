use actix_web::{HttpRequest, post, HttpResponse, patch, delete, get, web::{self, Json}};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Serialize, Deserialize};
use utoipa::openapi::security::HttpAuthScheme;

use crate::{utils::Role, repositories::{user::{UserRepository, UserModel}, token::TokenRepository}, error::{Result, OutsideError}, handlers::auth::verify_token};

fn to_json<T>(val: T) ->  Result<web::Json<T>> {
    Ok(web::Json(val))
}




#[derive(Debug, Serialize, Deserialize)]
pub struct PostUserRequest {
    name: String,
    email: String,
    password: String,
    requested_role: Role,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostUserResponse {
    id: String,
    name: String,
    email: String,
}

#[utoipa::path(
    params(
        ("data" = PostUserRequest,)
    ),
    responses(
        (status = 200, description = "Authorized user's token", body = PostUserResponse)
    )
)]
#[post("/api/users")]
pub async fn post_user(
    Json(data): web::Json<PostUserRequest>,
    repo: web::Data<UserRepository>,
) -> Result<web::Json<PostUserResponse>> {
    let user = UserModel::new(data.email, data.password, data.name);

    if !repo.create(&user).await?  {
        Err(OutsideError::NotUniqueEmail)?;
    };

    to_json(PostUserResponse { id: user.id.to_hex(), name: user.name, email: user.email })
}





#[derive(Debug, Serialize, Deserialize)]
pub struct PatchUserRequest {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
}


#[utoipa::path(
    params(
        ("data" = PatchUserRequest,)
    ),
    responses(
        (status = 200)
    )
)]
#[patch("/api/users")]
pub async fn patch_user(
    req: HttpRequest,
    Json(data): web::Json<PatchUserRequest>,
    users_repo: web::Data<UserRepository>,
    tokens_repo: web::Data<TokenRepository>,
) -> Result<HttpResponse> {
    let Ok((_, session)) = verify_token(&req, &tokens_repo).await? else {
        return Ok(HttpResponse::Unauthorized().finish()); //TODO: error description
    };

    let user_id = ObjectId::parse_str(&session.user_id).unwrap();

    let Some(mut user) = users_repo.find(user_id).await? else {
        return Ok(HttpResponse::ExpectationFailed().body("Error: User not found"));
    };

    if let Some(new_name) = data.name {
        user.name = new_name;
    }

    if let Some(new_email) = data.email {
        user.email = new_email;
    }

    if let Some(new_password) = data.password {
        user.password = new_password;
    }

    if &user.email != &session.email && users_repo.find_by_email(&user.email).await?.is_some() {
        return Ok(HttpResponse::BadRequest().body("Error: this email is already taken"));
    }  

    users_repo.delete(user_id).await?;

    users_repo.create(&user).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Authorized user's token", body = String)
    )
)]
#[delete("/api/users")]
pub async fn delete_user(
    req: HttpRequest,
    users_repo: web::Data<UserRepository>,
    tokens_repo: web::Data<TokenRepository>,
) -> Result<HttpResponse> {
    let Ok((_, session)) = verify_token(&req, &tokens_repo).await? else {
        return Ok(HttpResponse::Unauthorized().finish()); //TODO: error description
    };

    let user_id = ObjectId::parse_str(&session.user_id).unwrap();

    if users_repo.delete(user_id).await?.is_none() {
        return Ok(HttpResponse::BadRequest().body("Error: User not found"));
    }
    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUsersRequest {
    page: u32,
    limit: u32,
}

#[utoipa::path(
    responses(
        (status = 200, description = "Authorized user's token", body = String)
    )
)]
#[get("/api/users/")]
pub async fn get_users(Json(data): web::Json<GetUsersRequest>, repo: web::Data<UserRepository>) -> Result<HttpResponse> {
    let users = repo.users((data.page - 1) * data.limit, data.limit).await?;
    
    let users = users.into_iter().map(|user| doc!{"name": user.name, "email": user.email}).collect::<Vec<_>>();
    
    Ok(HttpResponse::Ok().json(doc!{"data": users, "hasNextPage": true, "page": data.page, "limit": data.limit}))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Authorized user's token", body = String)
    )
)]
#[get("/api/users/{email}")]
pub async fn get_user(
    email: web::Path<(String,)>,
    repo: web::Data<UserRepository>,
) -> Result<HttpResponse> {
    let (email, ) = email.into_inner();

    let Some(user) = repo.find_by_email(&email).await? else {
        return Ok(HttpResponse::BadRequest().body("Error: User not found"));
    };

    Ok(HttpResponse::Ok().json(doc!{"id": user.id, "name": user.name, "email": user.email}))
}
