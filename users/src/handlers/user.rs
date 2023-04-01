use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use common::{
    auth_session::{AuthSessionManager, SessionManager},
    entities::user::User,
};
use mongodb::bson::{doc, oid::ObjectId};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::str;
use utoipa::ToSchema;

use crate::{
    error::{Error, OuterError, Result},
    handlers::auth::verify_token,
    repositories::{token::TokenRepo, user::UserRepo},
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserView {
    id: String,
    name: String,
    email: String,
    current_role: String,
}

impl From<User<String>> for UserView {
    fn from(value: User<String>) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
            current_role: value.current_role,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub current_role: String,
}

#[utoipa::path(
    request_body(
        content = PostUserRequest
    ),
    responses(
        (status = 200, body = UserView)
    )
)]
#[post("/api/users")]
pub async fn post_user(
    Json(mut data): web::Json<PostUserRequest>,
    repo: web::Data<UserRepo>,
) -> Result<web::Json<UserView>> {
    let salt: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    data.password.push_str(&salt);
    let password = sha256::digest(data.password);
    let user = User {
        id: ObjectId::new(),
        name: data.name,
        email: data.email,
        salt,
        password,
        current_role: data.current_role,
        last_modified: Utc::now().timestamp_micros(),
    };

    if repo.find_by_email(&user.email).await?.is_some() {
        Err(Error::Outer(OuterError::NotUniqueEmail))?;
    };

    repo.create(&user).await?;
    return Ok(web::Json(user.stringify().into()));
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatchUserRequest {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
    current_role: Option<String>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PatchUserRequest
    ),
    responses(
        (status = 200, body = UserView)
    )
)]
#[patch("/api/users")]
pub async fn patch_user(
    req: HttpRequest,
    Json(data): web::Json<PatchUserRequest>,
    users_repo: web::Data<UserRepo>,
    tokens_repo: web::Data<TokenRepo>,
) -> Result<web::Json<UserView>> {
    let Ok((_, session)) = verify_token(&req, &tokens_repo).await? else {
        return Err(Error::Outer(OuterError::Unauthorized));
    };

    let user_id = session.user_id();

    let Some(mut user) = users_repo.find(user_id).await? else {
        return Err(Error::Outer(OuterError::UserNotFound));
    };

    if let Some(new_name) = data.name {
        user.name = new_name;
    }

    if let Some(mut new_password) = data.password {
        new_password.push_str(&user.salt);
        user.password = sha256::digest(new_password);
    }

    if let Some(new_role) = data.current_role {
        user.current_role = new_role;
    }

    if let Some(email) = data.email {
        if &user.email != &email && users_repo.find_by_email(&user.email).await?.is_some() {
            return Err(Error::Outer(OuterError::NotUniqueEmail));
        }
    }

    users_repo.delete(&user_id).await?;

    users_repo.create(&user).await?;

    Ok(web::Json(user.stringify().into()))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = UserView)
    )
)]
#[delete("/api/users")]
pub async fn delete_user(
    req: HttpRequest,
    users_repo: web::Data<UserRepo>,
    tokens_repo: web::Data<TokenRepo>,
) -> Result<web::Json<Option<UserView>>> {
    let Ok((_, session)) = verify_token(&req, &tokens_repo).await? else {
        return Err(Error::Outer(OuterError::Unauthorized)); //TODO: error description
    };

    let user_id = session.user_id();

    let Some(user) = users_repo.delete(&user_id).await? else {
        return Ok(web::Json(None));
    };
    Ok(web::Json(Some(user.stringify().into())))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetUsersRequest {
    page: u32,
    limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetUsersResponse {
    data: Vec<UserView>,
}

#[utoipa::path(
    request_body(
        content = GetUsersRequest,
    ),
    responses(
        (status = 200, body = GetUsersResponse)
    )
)]
#[get("/api/users/all")]
pub async fn get_users(
    Json(data): web::Json<GetUsersRequest>,
    repo: web::Data<UserRepo>,
) -> Result<HttpResponse> {
    let users = repo
        .find_all((data.page - 1) * data.limit, data.limit)
        .await?;

    Ok(HttpResponse::Ok().json(GetUsersResponse {
        data: users
            .into_iter()
            .map(|user| user.stringify().into())
            .collect(),
    }))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = UserView)
    )
)]
#[get("/api/users")]
pub async fn get_user(
    req: HttpRequest,
    repo: web::Data<UserRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager
        .get_session(req.into())
        .await
        .map_err(|_| Error::Outer(OuterError::Unauthorized))?;

    let Some(user) = repo.find(session.user_id()).await.unwrap() else {
        return Err(Error::Outer(OuterError::UserNotFound));
    };

    Ok(HttpResponse::Ok().json(user.stringify()))
}

#[cfg(test)]
mod test {
    use actix_web::test::{self, init_service};
    use common::{
        auth_session::{AuthSession, AuthSessionManager, Role, TestSessionManager},
        repository::test_repository::TestRepository,
    };
    use mongodb::bson::oid::ObjectId;

    use crate::{
        create_app, create_test_app,
        repositories::{list_element::ListElementRepository, token::TokenRepo, user::UserRepo},
    };

    #[actix_web::test]
    async fn test_post_user() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            role: Role::User,
            exp: 100000000,
        };
        let mut app = init_service(create_test_app(test_user)).await;

        let req = test::TestRequest::post()
            .uri("/api/users")
            .set_json(&super::PostUserRequest {
                name: "test".to_string(),
                email: "test@gmail.com".to_string(),
                password: "test".to_string(),
                current_role: "Customer".to_string(),
            })
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_post_user_with_existing_email() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            role: Role::User,
            exp: 100000000,
        };

        let user_repo = UserRepo::new(TestRepository::new());

        let token_repo = TokenRepo::new(TestRepository::new());

        let elem_repo = ListElementRepository::new(TestRepository::new());

        let test_manager = AuthSessionManager::new(TestSessionManager(test_user));

        let mut app =
            init_service(create_app(user_repo, token_repo, elem_repo, test_manager)).await;

        let req = test::TestRequest::post()
            .uri("/api/users")
            .set_json(&super::PostUserRequest {
                name: "test".to_string(),
                email: "test@gmail.com".to_string(),
                password: "test".to_string(),
                current_role: "Customer".to_string(),
            })
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let req = test::TestRequest::post()
            .uri("/api/users")
            .set_json(&super::PostUserRequest {
                name: "test".to_string(),
                email: "test@gmail.com".to_string(),
                password: "test".to_string(),
                current_role: "Customer".to_string(),
            })
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_delete_user() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            role: Role::User,
            exp: 100000000,
        };
        let mut app = init_service(create_test_app(test_user)).await;

        let req = test::TestRequest::post()
            .uri("/api/users")
            .set_json(&super::PostUserRequest {
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
            .set_json(&crate::LoginRequest {
                email: "test@gmail.com".to_string(),
                password: "test".to_string(),
            })
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let token = serde_json::from_slice::<crate::LoginResponse>(&body)
            .unwrap()
            .token;

        let req = test::TestRequest::delete()
            .uri("/api/users")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }
}
