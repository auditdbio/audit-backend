pub mod access_rules;
pub mod api;
pub mod auth;
pub mod constants;
pub mod context;
pub mod entities;
pub mod error;
pub mod repository;
pub mod services;
pub mod verification;

pub fn default_timestamp() -> i64 {
    chrono::Utc::now().timestamp_micros()
}
