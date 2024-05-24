use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::{
        audit_request::{AuditRequest, TimeRange},
        auditor::PublicAuditor,
        contacts::Contacts,
        project::PublicProject,
        role::Role,
    },
    error,
    services::{API_PREFIX, AUDITORS_SERVICE, AUDITS_SERVICE, CUSTOMERS_SERVICE, PROTOCOL},
};

#[derive(Clone, Deserialize, Serialize, Debug)]
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
    pub tags: Option<Vec<String>>,
    pub price: Option<i64>,
    pub total_cost: Option<i64>,
    pub auditor_contacts: Contacts,
    pub customer_contacts: Contacts,
    pub last_changer: Role,
}

impl PublicRequest {
    pub async fn new(
        context: &GeneralContext,
        request: AuditRequest<ObjectId>,
    ) -> error::Result<PublicRequest> {
        let auth = context.auth();
        let project = if let Ok(project) = context
            .make_request::<PublicProject>()
            .get(format!(
                "{}://{}/{}/project/{}",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str(),
                API_PREFIX.as_str(),
                request.project_id
            ))
            .auth(auth)
            .send()
            .await?
            .json::<PublicProject>()
            .await
        {
            project
        } else {
            context
                .make_request::<()>()
                .post(format!(
                    "{}://{}/{}/project/auditor/{}/{}",
                    PROTOCOL.as_str(),
                    CUSTOMERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    request.project_id,
                    request.auditor_id
                ))
                .auth(context.server_auth())
                .send()
                .await?;

            context
                .make_request::<PublicProject>()
                .get(format!(
                    "{}://{}/{}/project/{}",
                    PROTOCOL.as_str(),
                    CUSTOMERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    request.project_id
                ))
                .auth(auth)
                .send()
                .await?
                .json::<PublicProject>()
                .await
                .map_err(|_| anyhow::anyhow!("Project {} not found", request.project_id))?
        };

        let auditor = context
            .make_request::<PublicAuditor>()
            .auth(context.server_auth())
            .get(format!(
                "{}://{}/{}/auditor/{}",
                PROTOCOL.as_str(),
                AUDITORS_SERVICE.as_str(),
                API_PREFIX.as_str(),
                request.auditor_id
            ))
            .auth(auth)
            .send()
            .await?
            .json::<PublicAuditor>()
            .await
            .map_err(|_| anyhow::anyhow!("Auditor {} not found", request.auditor_id))?;

        let tags = if let Some(tags) = request.tags {
            tags
        } else {
            project.tags
        };

        let project_scope = if let Some(scope) = request.scope {
            scope
        } else {
            project.scope
        };

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
            project_scope,
            tags: Some(tags),
            price: request.price,
            total_cost: request.total_cost,
            auditor_contacts: auditor.contacts,
            customer_contacts: project.creator_contacts,
            last_changer: request.last_changer,
        })
    }
}

pub async fn get_audit_requests(
    context: &GeneralContext,
    auth: Auth,
) -> error::Result<Vec<PublicRequest>> {
    Ok(context
        .make_request::<Vec<PublicRequest>>()
        .get(format!(
            "{}://{}/{}/my_audit_request/auditor",
            PROTOCOL.as_str(),
            AUDITS_SERVICE.as_str(),
            API_PREFIX.as_str(),
        ))
        .auth(auth)
        .send()
        .await?
        .json::<Vec<PublicRequest>>()
        .await?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRequest {
    pub customer_id: String,
    pub auditor_id: String,
    pub project_id: String,

    pub price: Option<i64>,
    pub total_cost: Option<i64>,
    pub description: String,
    pub time: TimeRange,
}

pub async fn create_request(
    context: &GeneralContext,
    auth: Auth,
    data: CreateRequest,
) -> error::Result<()> {
    context
        .make_request::<CreateRequest>()
        .post(format!(
            "{}://{}/{}/audit_request",
            PROTOCOL.as_str(),
            AUDITORS_SERVICE.as_str(),
            API_PREFIX.as_str(),
        ))
        .auth(auth)
        .json(&data)
        .send()
        .await?;

    Ok(())
}

pub async fn delete(context: &GeneralContext, auth: Auth, id: ObjectId) -> error::Result<()> {
    context
        .make_request::<()>()
        .delete(format!(
            "{}://{}/{}/audit_request/{}",
            PROTOCOL.as_str(),
            AUDITORS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            id.to_hex()
        ))
        .auth(auth)
        .send()
        .await?;

    Ok(())
}
