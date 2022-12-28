use actix_web::{HttpRequest, post, HttpResponse, patch, delete, get, web::{self, Json}};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Serialize, Deserialize};

use crate::{utils::Role, repositories::{user::{UserRepository, UserModel}, token::TokenRepository}, internal_error};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostUserRequest {
    name: String,
    email: String,
    password: String,
    requested_role: Role,
}

// QUESTION: should I immediately login?
#[post("/api/users")]
pub async fn post_user(
    req: HttpRequest, 
    Json(data): web::Json<PostUserRequest>,
    repo: web::Data<UserRepository>,
) -> HttpResponse {
    let user = UserModel {
        id: None,
        email: data.email,
        password: data.password,
        name: data.name,
    };

    let Some(user) = internal_error!(repo.create(user).await) else {
        return HttpResponse::BadRequest().body("Error: Email is not unique");
    };

    HttpResponse::Ok().json(doc!{"id": user.id, "name": user.name, "email": user.email, "status": "Active"}) // Is status always active?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchUserRequest {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

#[patch("/api/users")]
pub async fn patch_user(
    req: HttpRequest,
    Json(data): web::Json<PatchUserRequest>,
    users_repo: web::Data<UserRepository>,
    tokens_repo: web::Data<TokenRepository>,
) -> HttpResponse {
    let Ok((_, claims)) = internal_error!(super::auth::verify_token(&req, &tokens_repo).await) else {
        return HttpResponse::Unauthorized().finish(); //TODO: error description
    };

    let user_id = ObjectId::parse_str(&claims.user_id).unwrap();

    let Some(mut user) = internal_error!(users_repo.find(user_id).await) else {
        return HttpResponse::ExpectationFailed().body("Error: User not found");
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

    if &user.email != &claims.email && internal_error!(users_repo.find_by_email(&user.email).await).is_some() {
        return HttpResponse::BadRequest().body("Error: this email is already taken");
    }  

    internal_error!(users_repo.delete(user_id).await);

    internal_error!(users_repo.create(user).await).unwrap();

    HttpResponse::Ok().finish()
}


#[delete("/api/users")]
pub async fn delete_user(
    req: HttpRequest,
    users_repo: web::Data<UserRepository>,
    tokens_repo: web::Data<TokenRepository>,
) -> HttpResponse {
    let Ok((_, claims)) = internal_error!(super::auth::verify_token(&req, &tokens_repo).await) else {
        return HttpResponse::Unauthorized().finish(); //TODO: error description
    };

    let user_id = ObjectId::parse_str(&claims.user_id).unwrap();

    if internal_error!(users_repo.delete(user_id).await).is_none() {
        return HttpResponse::BadRequest().body("Error: User not found");
    }
    HttpResponse::Ok().finish()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUsersRequest {
    page: u32,
    limit: u32,
}

#[get("/api/users/{email}")]
pub async fn get_users(Json(data): web::Json<GetUsersRequest>, repo: web::Data<UserRepository>) -> HttpResponse {
    let users = internal_error!(repo.users(data.page, data.limit).await);
    let users = users.into_iter().map(|user| doc!{"name": user.name, "email": user.email}).collect::<Vec<_>>();
    HttpResponse::Ok().json(doc!{"data": users, "hasNextPage": true, "page": data.page, "limit": data.limit})
}

#[get("/api/users/{email}")]
pub async fn get_user(
    email: web::Path<(String,)>,
    repo: web::Data<UserRepository>,
) -> HttpResponse {
    let (email, ) = email.into_inner();

    let Some(user) = internal_error!(repo.find_by_email(&email).await) else {
        return HttpResponse::BadRequest().body("Error: User not found");
    };

    HttpResponse::Ok().json(doc!{"id": user.id, "name": user.name, "email": user.email, "status": "Active"}) // Is status always active?
}
