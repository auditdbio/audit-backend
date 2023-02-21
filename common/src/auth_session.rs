use actix_web::HttpRequest;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::error::Error;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

pub async fn get_auth_session(req: &HttpRequest) -> Result<AuthSession, Box<dyn Error>> {
    let client = awc::Client::default();

    let jwt = jwt_from_header(req).unwrap();

    let req = client
        .get("http://127.0.0.1:8080")
        .insert_header(("Authorization", format!("Bearer {}", jwt)));
    let mut res = req.send().await?;

    let body = res.json::<AuthSession>().await?;
    Ok(body)
}
