use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LinkedService {
    GitHub,
    Gitcoin,
    Facebook,
    X,
    LinkedIn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetGithubAccessToken {
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubAccessResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubUserData {
    pub id: i32,
    pub login: String,
    pub name: Option<String>,
    pub html_url: String,
    pub avatar_url: String,
    pub company: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubUserEmails {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetXAccessToken {
    pub code: String,
    pub client_id: String,
    pub grant_type: String,
    pub redirect_uri: String,
    pub code_verifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XAccessResponse {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUserResponse {
    pub data: XUserData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUserData {
    pub id: String,
    pub name: String,
    pub username: String,
    pub profile_image_url: Option<String>,
    pub url: Option<String>,
}
