use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use common::{entities::user::User, auth_session::get_auth_session};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    error::{Error, OuterError, Result},
    handlers::auth::verify_token,
    repositories::{token::TokenRepository, user::UserRepository},
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostUserRequest {
    name: String,
    email: String,
    password: String,
    current_role: String,
}

#[utoipa::path(
    request_body(
        content = PostUserRequest
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
        current_role: data.current_role,
    };

    if !repo.create(&user).await? {
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
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
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
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetUsersResponse {
    data: Vec<User>,
    has_next_page: bool,
    page: u32,
    limit: u32,
}

#[utoipa::path(
    request_body(
        content = GetUsersRequest,
    ),
    responses(
        (status = 200, description = "Authorized user's token", body = GetUsersResponse)
    )
)]
#[get("/api/users")]
pub async fn get_users(
    Json(data): web::Json<GetUsersRequest>,
    repo: web::Data<UserRepository>,
) -> Result<HttpResponse> {
    let users = repo.users((data.page - 1) * data.limit, data.limit).await?;

    Ok(HttpResponse::Ok().json(GetUsersResponse {
        data: users,
        has_next_page: false,
        page: data.page,
        limit: data.limit,
    }))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, description = "Authorized user's token", body = User)
    )
)]
#[get("/api/users/user")]
pub async fn get_user(
    req: HttpRequest,
    repo: web::Data<UserRepository>,
) -> Result<HttpResponse> {    
    let session = get_auth_session(&req).await.unwrap();

    let user = repo.find(session.user_id()).await.unwrap();

    Ok(HttpResponse::Ok().json(user))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Authorized user's token", body = String)
    )
)]
#[get("/api/users/{email}")]
pub async fn get_user_email(
    email: web::Path<(String,)>,
    repo: web::Data<UserRepository>,
) -> Result<HttpResponse> {
    let (email,) = email.into_inner();

    let Some(user) = repo.find_by_email(&email).await? else {
        return Ok(HttpResponse::BadRequest().body("Error: User not found"));
    };

    Ok(HttpResponse::Ok().json(doc! {"id": user.id, "name": user.name, "email": user.email}))
}

// #[cfg(test)]
// mod test {
//     use actix_web::{test::{self, init_service}, App};

//     #[actix_web::test]
//     async fn test_post_user() {
//         let req = test::TestRequest::post()
//             .uri("/api/users")
//             .set_json(&super::PostUserRequest {
//                 name: "test".to_string(),
//                 email: "test@gmail.com".to_string(),
//                 password: "test".to_string(),
//             })
//             .to_request();

//         let mut app = init_service(App::new().configure(crate::configure_service)).await;
//         let resp = test::call_service(&mut app, req).await;
//         assert!(resp.status().is_success());
//     }
// }
