use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{
        audit_request::{AuditRequest, PriceRange, TimeRange},
        auditor::PublicAuditor,
        contacts::Contacts,
        project::PublicProject,
        role::Role,
    },
    error::{self, AddCode},
    services::{AUDITORS_SERVICE, CUSTOMERS_SERVICE, PROTOCOL},
};

use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRequest {
    customer_id: String,
    auditor_id: String,
    project_id: String,

    price: i64,
    description: String,
    time: TimeRange,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestChange {
    description: Option<String>,
    time: Option<TimeRange>,
    project_scope: Option<Vec<String>>,
    price_range: Option<PriceRange>,
    price: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicRequest {
    pub id: String,
    pub auditor_first_name: String,
    pub auditor_last_name: String,
    pub customer_id: String,
    pub auditor_id: String,
    pub project_id: String,
    pub description: String,
    pub time: TimeRange,
    pub project_name: String,
    pub avatar: String,
    pub project_scope: Vec<String>,
    pub price: i64,
    pub auditor_contacts: Contacts,
    pub customer_contacts: Contacts,
}

impl PublicRequest {
    pub async fn new(
        context: &Context,
        request: AuditRequest<ObjectId>,
    ) -> error::Result<PublicRequest> {
        let project = context
            .make_request::<PublicProject>()
            .auth(context.server_auth())
            .get(format!(
                "{}://{}/api/project/{}",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str(),
                request.project_id
            ))
            .auth(context.server_auth())
            .send()
            .await?
            .json::<PublicProject>()
            .await?;

        let auditor = context
            .make_request::<PublicAuditor>()
            .auth(context.server_auth())
            .get(format!(
                "{}://{}/api/auditor/{}",
                PROTOCOL.as_str(),
                AUDITORS_SERVICE.as_str(),
                request.auditor_id
            ))
            .auth(context.server_auth())
            .send()
            .await?
            .json::<PublicAuditor>()
            .await?;

        Ok(PublicRequest {
            id: request.id.to_hex(),
            customer_id: request.customer_id.to_hex(),
            auditor_id: request.auditor_id.to_hex(),
            project_id: request.project_id.to_hex(),
            auditor_first_name: auditor.first_name,
            auditor_last_name: auditor.last_name,
            description: request.description,
            time: request.time,
            project_name: project.name,
            avatar: auditor.avatar,
            project_scope: project.scope,
            price: request.price,
            auditor_contacts: auditor.contacts,
            customer_contacts: project.creator_contacts,
        })
    }
}

pub struct RequestService {
    context: Context,
}

impl RequestService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, request: CreateRequest) -> error::Result<PublicRequest> {
        let auth = self.context.auth();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(user_id) = auth.id() else {
            return Err(anyhow::anyhow!("Audit can be created only by authenticated user").code(400));
        };

        let customer_id = request.customer_id.parse()?;
        let auditor_id = request.auditor_id.parse()?;

        if &customer_id == &auditor_id {
            return Err(anyhow::anyhow!("You can't create audit with yourself").code(400));
        }

        let last_changer = if user_id == &customer_id {
            Role::Customer
        } else if user_id == &auditor_id {
            Role::Auditor
        } else {
            return Err(
                anyhow::anyhow!("Audit can be created only by customer or auditor").code(400),
            );
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
            .auth(self.context.server_auth())
            .send()
            .await?
            .json::<PublicProject>()
            .await?;

        let request = AuditRequest {
            id: ObjectId::new(),
            customer_id,
            auditor_id,
            project_id: request.project_id.parse()?,
            description: request.description,
            time: request.time,
            project_scope: project.scope,
            price: request.price,
            last_modified: Utc::now().timestamp_micros(),
            last_changer,
        };

        let old_version_of_this_request = requests
            .find_many("project_id", &Bson::ObjectId(request.project_id))
            .await?
            .into_iter()
            .filter(|r| r.customer_id == request.customer_id && r.auditor_id == request.auditor_id)
            .collect::<Vec<_>>()
            .pop();

        if let Some(old_version_of_this_request) = old_version_of_this_request {
            requests
                .delete("id", &old_version_of_this_request.id)
                .await?;
        }

        requests.insert(&request).await?;

        let public_request = PublicRequest::new(&self.context, request).await?;

        Ok(public_request)
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicRequest>> {
        let auth = self.context.auth();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(request) = requests.find("id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(auth, &request) {
            return Err(anyhow::anyhow!("User is not available to change this customer").code(400));
        }

        let public_request = PublicRequest::new(&self.context, request).await?;

        Ok(Some(public_request))
    }

    pub async fn my_request(&self, role: Role) -> error::Result<Vec<PublicRequest>> {
        let auth = self.context.auth();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(user_id) = auth.id() else {
            return Err(anyhow::anyhow!("Audit can be created only by authenticated user").code(400));
        };

        let id = match role {
            Role::Auditor => "auditor_id",
            Role::Customer => "customer_id",
        };

        let result = requests
            .find_many(id, &Bson::ObjectId(user_id.clone()))
            .await?;

        let mut public_requests = Vec::new();

        for req in result {
            let public_request = PublicRequest::new(&self.context, req).await?;

            public_requests.push(public_request);
        }

        Ok(public_requests)
    }

    pub async fn change(
        &self,
        id: ObjectId,
        change: RequestChange,
    ) -> error::Result<PublicRequest> {
        let auth = self.context.auth();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(mut request) = requests.find("id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No customer found").code(404));
        };

        if !Edit.get_access(auth, &request) {
            return Err(anyhow::anyhow!("User is not available to change this customer").code(400));
        }

        if let Some(description) = change.description {
            request.description = description;
        }

        if let Some(time) = change.time {
            request.time = time;
        }

        if let Some(project_scope) = change.project_scope {
            request.project_scope = project_scope;
        }

        if let Some(price) = change.price {
            request.price = price;
        }

        let role = if auth.id() == Some(&request.customer_id) {
            Role::Customer
        } else if auth.id() == Some(&request.auditor_id) {
            Role::Auditor
        } else {
            return Err(anyhow::anyhow!("User is not available to change this customer").code(400));
        };

        request.last_changer = role;

        request.last_modified = Utc::now().timestamp_micros();

        requests.delete("id", &id).await?;
        requests.insert(&request).await?;

        let public_request = PublicRequest::new(&self.context, request).await?;

        Ok(public_request)
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<PublicRequest> {
        let auth = self.context.auth();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(request) = requests.delete("id", &id).await? else {
            return Err(anyhow::anyhow!("No customer found").code(404));
        };

        if !Edit.get_access(auth, &request) {
            requests.insert(&request).await?;
            return Err(anyhow::anyhow!("User is not available to delete this customer").code(400));
        }

        let public_request = PublicRequest::new(&self.context, request).await?;

        Ok(public_request)
    }
}
