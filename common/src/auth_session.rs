use actix_web::{dev::Payload, HttpRequest};
use awc::error::SendRequestError;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{error::Error, f32::consts::E};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct AuthSession {
    pub user_id: String,
    pub token: String,
    pub exp: usize,
}

impl AuthSession {
    pub fn user_id(&self) -> ObjectId {
        ObjectId::parse_str(&self.user_id).unwrap()
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
        .get("http://127.0.0.1:8080")
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

pub struct TestSessionManager(AuthSession);

#[async_trait::async_trait]
impl SessionManager for TestSessionManager {
    type Error = String;
    type Payload = AuthPayload;

    async fn get_session(&self, _req: Self::Payload) -> Result<AuthSession, Self::Error> {
        Ok(self.0.clone())
    }
}

pub struct AuthSessionManager(
    Box<dyn SessionManager<Error = String, Payload = AuthPayload> + Send + Sync>,
);

#[async_trait::async_trait]
impl SessionManager for AuthSessionManager {
    type Error = String;
    type Payload = AuthPayload;

    async fn get_session(&self, req: Self::Payload) -> Result<AuthSession, Self::Error> {
        self.0.get_session(req).await
    }
}
