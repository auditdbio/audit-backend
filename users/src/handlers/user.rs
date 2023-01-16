use actix_web::{HttpRequest, post, HttpResponse, patch, delete, get, web::{self, Json}};
use common::entities::user::User;
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::{repositories::{user::{UserRepository}, token::TokenRepository}, error::{Result, Error, OuterError}, handlers::auth::verify_token};





#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostUserRequest {
    name: String,
    email: String,
    password: String,
}

#[utoipa::path(
    request_body(
        content = LoginRequest
    ),
    responses(
        (status = 200, description = "Authorized user's token", body = User)
    )
)]
#[post("/api/users")]
pub async fn post_user(
    Json(data): web::Json<PostUserRequest>,
    repo: web::Data<UserRepository>,
) -> Result<web::Json<User>> {
    let user = User {
        id: ObjectId::new(),
        name: data.name,
        email: data.email,
        password: data.password,
    };

    if !repo.create(&user).await?  {
        Err(Error::Outer(OuterError::NotUniqueEmail))?;
    };

    return Ok(web::Json(user));
}





#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatchUserRequest {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

#[utoipa::path(
    request_body(
        content = LoginRequest
    ),
    responses(
        (status = 200, body = PublicUser)
    )
)]
#[patch("/api/users")]
pub async fn patch_user(
    req: HttpRequest,
    Json(data): web::Json<PatchUserRequest>,
    users_repo: web::Data<UserRepository>,
    tokens_repo: web::Data<TokenRepository>,
) -> Result<web::Json<User>> {
    let Ok((_, session)) = verify_token(&req, &tokens_repo).await? else {
        return Err(Error::Outer(OuterError::Unauthorized));
    };

    let user_id = ObjectId::parse_str(&session.user_id).unwrap();

    let Some(mut user) = users_repo.find(user_id).await? else {
        return Err(Error::Outer(OuterError::UserNotFound));
    };

    if let Some(new_name) = data.name {
        user.name = new_name;
    }

    if let Some(new_password) = data.password {
        user.password = new_password;
    }

    if let Some(email) = data.email {

        if &user.email != &email && users_repo.find_by_email(&user.email).await?.is_some() {
            return Err(Error::Outer(OuterError::NotUniqueEmail));
        }  
    }

    users_repo.delete(user_id).await?;

    users_repo.create(&user).await?;

    Ok(web::Json(user))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Authorized user's token", body = User)
    )
)]
#[delete("/api/users")]
pub async fn delete_user(
    req: HttpRequest,
    users_repo: web::Data<UserRepository>,
    tokens_repo: web::Data<TokenRepository>,
) -> Result<web::Json<User>> {
    let Ok((_, session)) = verify_token(&req, &tokens_repo).await? else {
        return Err(Error::Outer(OuterError::Unauthorized)); //TODO: error description
    };

    let user_id = ObjectId::parse_str(&session.user_id).unwrap();

    let deleted_user = users_repo.delete(user_id).await?;
    if !deleted_user.is_none() {
        return Err(Error::Outer(OuterError::UserNotFound));
    }
    Ok(web::Json(deleted_user.unwrap()))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
