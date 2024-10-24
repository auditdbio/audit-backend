use std::collections::HashMap;

use common::{
    api::events::{post_event, EventPayload, PublicEvent},
    context::GeneralContext,
    error,
    services::{
        AUDITORS_SERVICE, AUDITS_SERVICE, CUSTOMERS_SERVICE, FILES_SERVICE, MAIL_SERVICE,
        NOTIFICATIONS_SERVICE, PROTOCOL, SEARCH_SERVICE, USERS_SERVICE,
    },
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Activity {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Status(HashMap<String, Activity>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    ping: String,
    name: String,
}

impl Service {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            ping: format!("{}://api/{}/{}/ping", PROTOCOL.as_str(), url, name),
            name: name.to_string(),
        }
    }
}

pub async fn status(context: GeneralContext, services: &Vec<Service>) -> Status {
    let mut result = HashMap::new();
    for service in services {
        let response = context
            .make_request::<()>()
            .get(service.ping.clone())
            .auth(context.server_auth())
            .send()
            .await;

        let activity = match response {
            Ok(_) => Activity::Up,
            Err(_) => Activity::Down,
        };

        result.insert(service.name.clone(), activity);
    }

    Status(result)
}

pub fn services() -> Vec<Service> {
    vec![
        Service::new("auditors", AUDITORS_SERVICE.as_str()),
        Service::new("audits", AUDITS_SERVICE.as_str()),
        Service::new("customers", CUSTOMERS_SERVICE.as_str()),
        Service::new("files", FILES_SERVICE.as_str()),
        Service::new("mail", MAIL_SERVICE.as_str()),
        Service::new("notifications", NOTIFICATIONS_SERVICE.as_str()),
        Service::new("search", SEARCH_SERVICE.as_str()),
        Service::new("users", USERS_SERVICE.as_str()),
    ]
}

pub async fn update(context: &GeneralContext) -> error::Result<()> {
    let event = PublicEvent::new(ObjectId::new(), None, EventPayload::VersionUpdate);
    post_event(context, event, context.server_auth()).await?;
    Ok(())
}
