use std::collections::HashMap;

use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{
        audit_request::{AuditRequest, PriceRange, TimeRange},
        auditor::PublicAuditor,
        project::PublicProject,
        role::Role,
    },
    services::{AUDITORS_SERVICE, CUSTOMERS_SERVICE, PROTOCOL},
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRequest {
    customer_id: String,
    auditor_id: String,
    project_id: String,

    description: String,
    time: TimeRange,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestChange {
    description: Option<String>,
    time: Option<TimeRange>,
    project_name: Option<String>,
    project_avatar: Option<String>,
    project_scope: Option<Vec<String>>,
    price_range: Option<PriceRange>,
    price: Option<i64>,
    auditor_contacts: Option<HashMap<String, String>>,
    customer_contacts: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicRequest {
    pub id: String,
    pub customer_id: String,
    pub auditor_id: String,
    pub project_id: String,
    pub description: String,
    pub time: TimeRange,
    pub project_name: String,
    pub project_avatar: String,
    pub project_scope: Vec<String>,
    pub price_range: Option<PriceRange>,
    pub price: Option<i64>,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
}

impl From<AuditRequest<ObjectId>> for PublicRequest {
    fn from(request: AuditRequest<ObjectId>) -> Self {
        Self {
            id: request.id.to_hex(),
            customer_id: request.customer_id.to_hex(),
            auditor_id: request.auditor_id.to_hex(),
            project_id: request.project_id.to_hex(),
            description: request.description,
            time: request.time,
            project_name: request.project_name,
            project_avatar: request.project_avatar,
            project_scope: request.project_scope,
            price_range: request.price_range,
            price: request.price,
            auditor_contacts: request.auditor_contacts,
            customer_contacts: request.customer_contacts,
        }
    }
}

pub struct RequestService {
    context: Context,
}

impl RequestService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, request: CreateRequest) -> anyhow::Result<PublicRequest> {
        let auth = self.context.auth();

        let Some(requests) = self.context.get_repository::<AuditRequest<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(user_id) = auth.id() else {
            bail!("Audit can be created only by authenticated user")
        };

        let customer_id = request.customer_id.parse()?;
        let auditor_id = request.auditor_id.parse()?;

        let last_changer = if user_id == &customer_id {
            Role::Customer
        } else if user_id == &auditor_id {
            Role::Auditor
        } else {
            bail!("Audit can be created only by customer or auditor")
        };

        let project = self
            .context
            .make_request::<PublicProject>()
            .auth(auth.clone())
            .get(format!(
                "{}://{}/api/project/{}",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str(),
                request.project_id
            ))
            .send()
            .await?
            .json::<PublicProject>()
            .await?;

        let auditor = self
            .context
            .make_request::<PublicAuditor>()
            .auth(auth.clone())
            .get(format!(
                "{}://{}/api/auditor/{}",
                PROTOCOL.as_str(),
                AUDITORS_SERVICE.as_str(),
                request.auditor_id
            ))
            .send()
            .await?
            .json::<PublicAuditor>()
            .await?;

        let request = AuditRequest {
            id: ObjectId::new(),
            customer_id,
            auditor_id,
            project_id: request.project_id.parse()?,
            description: request.description,
            time: request.time,
            project_name: project.name,
            project_avatar: auditor.avatar,
            project_scope: project.scope,
            price_range: Some(auditor.price_range),
            price: None,
            auditor_contacts: auditor.contacts,
            customer_contacts: project.creator_contacts,
            last_modified: Utc::now().timestamp_micros(),
            last_changer,
        };

        requests.insert(&request).await?;

        Ok(request.into())
    }

    pub async fn find(&self, id: ObjectId) -> anyhow::Result<Option<PublicRequest>> {
        let auth = self.context.auth();

        let Some(requests) = self.context.get_repository::<AuditRequest<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(request) = requests.find("id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read::get_access(auth, &request) {
            bail!("User is not available to change this customer")
        }

        Ok(Some(request.into()))
    }

    pub async fn my_request(&self, role: Role) -> anyhow::Result<Vec<AuditRequest<String>>> {
        let auth = self.context.auth();

        let Some(requests) = self.context.get_repository::<AuditRequest<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(user_id) = auth.id() else {
            bail!("Audit can be created only by authenticated user")
        };

        let id = match role {
            Role::Auditor => "auditor_id",
            Role::Customer => "customer_id",
        };

        let result = requests
            .find_many(id, &Bson::ObjectId(user_id.clone()))
            .await?
            .into_iter()
            .map(AuditRequest::stringify)
            .collect();

        Ok(result)
    }

    pub async fn change(
        &self,
        id: ObjectId,
        change: RequestChange,
    ) -> anyhow::Result<PublicRequest> {
        let auth = self.context.auth();

        let Some(requests) = self.context.get_repository::<AuditRequest<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(mut request) = requests.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No customer found")
        };

        if !Edit::get_access(auth, &request) {
            bail!("User is not available to change this customer")
        }

        if let Some(description) = change.description {
            request.description = description;
        }

        if let Some(time) = change.time {
            request.time = time;
        }

        if let Some(project_name) = change.project_name {
            request.project_name = project_name;
        }

        if let Some(project_avatar) = change.project_avatar {
            request.project_avatar = project_avatar;
        }

        if let Some(project_scope) = change.project_scope {
            request.project_scope = project_scope;
        }

        if let Some(price) = change.price {
            request.price = Some(price);
        }

        if let Some(price_range) = change.price_range {
            request.price_range = Some(price_range);
        }

        if let Some(auditor_contacts) = change.auditor_contacts {
            request.auditor_contacts = auditor_contacts;
        }

        if let Some(customer_contacts) = change.customer_contacts {
            request.customer_contacts = customer_contacts;
        }

        let role = if auth.id() == Some(&request.customer_id) {
            Role::Customer
        } else if auth.id() == Some(&request.auditor_id) {
            Role::Auditor
        } else {
            bail!("User is not available to change this customer")
        };

        request.last_changer = role;

        request.last_modified = Utc::now().timestamp_micros();

        requests.delete("id", &id).await?;
        requests.insert(&request).await?;

        Ok(request.into())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicRequest> {
        let auth = self.context.auth();

        let Some(requests) = self.context.get_repository::<AuditRequest<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(request) = requests.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No customer found")
        };

        if !Edit::get_access(auth, &request) {
            requests.insert(&request).await?;
            bail!("User is not available to delete this customer")
        }

        Ok(request.into())
    }
}
