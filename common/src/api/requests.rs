use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    entities::{
        audit_request::{AuditRequest, TimeRange},
        auditor::PublicAuditor,
        contacts::Contacts,
        project::PublicProject,
        role::Role,
    },
    error,
    services::{AUDITORS_SERVICE, CUSTOMERS_SERVICE, PROTOCOL},
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
    pub price: i64,
    pub auditor_contacts: Contacts,
    pub customer_contacts: Contacts,
    pub last_changer: Role,
}

impl PublicRequest {
    pub async fn new(
        context: &Context,
        request: AuditRequest<ObjectId>,
    ) -> error::Result<PublicRequest> {
        let auth = context.auth();
        let project = if let Ok(project) = context
            .make_request::<PublicProject>()
            .get(format!(
                "{}://{}/api/project/{}",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str(),
                request.project_id
            ))
            .auth(auth.clone())
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
                    "{}://{}/api/project/auditor/{}/{}",
                    PROTOCOL.as_str(),
                    CUSTOMERS_SERVICE.as_str(),
                    request.project_id,
                    request.auditor_id
                ))
                .auth(context.server_auth())
                .send()
                .await?;

            context
                .make_request::<PublicProject>()
                .get(format!(
                    "{}://{}/api/project/{}",
                    PROTOCOL.as_str(),
                    CUSTOMERS_SERVICE.as_str(),
                    request.project_id
                ))
                .auth(auth.clone())
                .send()
                .await?
                .json::<PublicProject>()
                .await?
        };

        let auditor = context
            .make_request::<PublicAuditor>()
            .auth(context.server_auth())
            .get(format!(
                "{}://{}/api/auditor/{}",
                PROTOCOL.as_str(),
                AUDITORS_SERVICE.as_str(),
                request.auditor_id
            ))
            .auth(auth.clone())
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
            last_changer: request.last_changer,
        })
    }
}
