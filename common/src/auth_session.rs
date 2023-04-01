use actix_web::HttpRequest;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::services::USERS_SERVICE;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq, Eq)]
pub enum Role {
    User,
    Admin,
    Service,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct AuthSession {
    pub user_id: ObjectId,
    pub token: String,
    pub exp: usize,
    pub role: Role,
}

impl AuthSession {
    pub fn user_id(&self) -> ObjectId {
        self.user_id.clone()
    }
}

pub fn jwt_from_header(req: &HttpRequest) -> Option<String> {
    // possibly make readable error
    req.headers()
        .get("Authorization")
        .and_then(|x| x.to_str().ok())
        .and_then(|x| x.strip_prefix("Bearer ")) // remove prefix
        .map(str::to_string)
}

pub async fn get_auth_session(jwt: String) -> Result<AuthSession, String> {
    let client = reqwest::Client::new();
    let req = client
        .get(format!(
            "http://{}/api/auth/verify",
            USERS_SERVICE.as_str()
        ))
        .header("Authorization", format!("Bearer {}", jwt))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<AuthSession>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(req)
}

pub struct AuthPayload {
    pub jwt: String,
}

impl From<HttpRequest> for AuthPayload {
    fn from(req: HttpRequest) -> Self {
        let jwt = jwt_from_header(&req).unwrap();
        Self { jwt }
    }
}

#[async_trait::async_trait]
pub trait SessionManager {
    type Error;
    type Payload: From<HttpRequest> + Send;
    async fn get_session(&self, req: Self::Payload) -> Result<AuthSession, Self::Error>;
}
pub struct HttpSessionManager;

#[async_trait::async_trait]
impl SessionManager for HttpSessionManager {
    type Error = String;
    type Payload = AuthPayload;

    async fn get_session(&self, req: Self::Payload) -> Result<AuthSession, Self::Error> {
        get_auth_session(req.jwt).await
    }
}

pub struct TestSessionManager(pub AuthSession);

#[async_trait::async_trait]
impl SessionManager for TestSessionManager {
    type Error = String;
    type Payload = AuthPayload;

    async fn get_session(&self, _req: Self::Payload) -> Result<AuthSession, Self::Error> {
        Ok(self.0.clone())
    }
}

#[derive(Clone)]
pub struct AuthSessionManager(
    Arc<dyn SessionManager<Error = String, Payload = AuthPayload> + Send + Sync>,
);

impl AuthSessionManager {
    pub fn new<T>(manager: T) -> Self
    where
        T: SessionManager<Error = String, Payload = AuthPayload> + Send + Sync + 'static,
    {
        Self(Arc::new(manager))
    }
}

#[async_trait::async_trait]
impl SessionManager for AuthSessionManager {
    type Error = String;
    type Payload = AuthPayload;

    async fn get_session(&self, req: Self::Payload) -> Result<AuthSession, Self::Error> {
        self.0.get_session(req).await
    }
}
