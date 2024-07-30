use std::env::var;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROTOCOL: String = var("PROTOCOL").unwrap();
    pub static ref API_PREFIX: String = var("API_PREFIX").unwrap();
    pub static ref FRONTEND: String = var("FRONTEND").unwrap();
    pub static ref AUDITORS_SERVICE: String = var("AUDITORS_SERVICE_URL").unwrap();
    pub static ref AUDITS_SERVICE: String = var("AUDITS_SERVICE_URL").unwrap();
    pub static ref CUSTOMERS_SERVICE: String = var("CUSTOMERS_SERVICE_URL").unwrap();
    pub static ref CHAT_SERVICE: String = var("CHAT_SERVICE_URL").unwrap();
    pub static ref FILES_SERVICE: String = var("FILES_SERVICE_URL").unwrap();
    pub static ref RENDERER_SERVICE: String = var("RENDERER_SERVICE_URL").unwrap();
    pub static ref REPORT_SERVICE: String = var("REPORT_SERVICE_URL").unwrap();
    pub static ref RATING_SERVICE: String = var("RATING_SERVICE_URL").unwrap();
    pub static ref MAIL_SERVICE: String = var("MAIL_SERVICE_URL").unwrap();
    pub static ref SEARCH_SERVICE: String = var("SEARCH_SERVICE_URL").unwrap();
    pub static ref NOTIFICATIONS_SERVICE: String = var("NOTIFICATIONS_SERVICE_URL").unwrap();
    pub static ref USERS_SERVICE: String = var("USERS_SERVICE_URL").unwrap();
    pub static ref EVENTS_SERVICE: String = var("EVENTS_SERVICE_URL").unwrap();
}
