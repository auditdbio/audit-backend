use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    entities::{
        audit::{Audit, AuditStatus, PublicAuditStatus},
        audit_request::TimeRange,
        auditor::PublicAuditor,
        contacts::Contacts,
        issue::Status,
        project::PublicProject,
    },
    error,
    services::{CUSTOMERS_SERVICE, PROTOCOL},
};

use super::issue::PublicIssue;

#[derive(Debug, Serialize, Deserialize)]
pub enum AuditAction {
    #[serde(alias = "start")]
    Start,
    #[serde(alias = "resolve")]
    Resolve,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditChange {
    pub avatar: Option<String>,
    pub action: Option<AuditAction>,
    pub scope: Option<Vec<String>>,
    pub report_name: Option<String>,
    pub report: Option<String>,
    pub time: Option<TimeRange>,
    pub start_audit: Option<bool>,
    #[serde(rename = "isPublic")]
    pub public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIssue {
    pub name: String,
    pub description: String,
    pub status: Status,
    pub severity: String,
    pub category: String,
    #[serde(default)]
    pub links: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PublicAudit {
    pub id: String,
    pub auditor_id: String,
    pub customer_id: String,
    pub project_id: String,
    #[serde(rename = "isPublic")]
    pub public: bool,

    pub auditor_first_name: String,
    pub auditor_last_name: String,

    pub project_name: String,
    pub avatar: String,
    pub description: String,
    pub status: PublicAuditStatus,
    pub scope: Vec<String>,
    pub price: i64,

    pub auditor_contacts: Contacts,
    pub customer_contacts: Contacts,
    pub tags: Vec<String>,
    pub last_modified: i64,
    pub report: Option<String>,
    pub report_name: Option<String>,
    pub time: TimeRange,

    pub issues: Vec<PublicIssue>,
}

impl PublicAudit {
    pub async fn new(context: &Context, audit: Audit<ObjectId>) -> error::Result<PublicAudit> {
        let auth = context.auth();
        let auditor = context
            .make_request::<PublicAuditor>()
            .get(format!(
                "{}://{}/api/auditor/{}",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str(),
                audit.auditor_id
            ))
            .auth(context.server_auth())
            .send()
            .await?
            .json::<PublicAuditor>()
            .await?;

        let project = context
            .make_request::<PublicProject>()
            .get(format!(
                "{}://{}/api/project/{}",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str(),
                audit.project_id
            ))
            .auth(context.server_auth())
            .send()
            .await?
            .json::<PublicProject>()
            .await?;

        let status = match audit.status {
            AuditStatus::Waiting => PublicAuditStatus::WaitingForAudit,
            AuditStatus::Started => {
                if audit.report.is_some() {
                    PublicAuditStatus::ReadyForResolve
                } else if audit.issues.is_empty() {
                    PublicAuditStatus::InProgress
                } else if audit.issues.iter().all(|issue| issue.is_resolved()) {
                    PublicAuditStatus::ReadyForResolve
                } else {
                    PublicAuditStatus::IssuesWorkflow
                }
            }
            AuditStatus::Resolved => PublicAuditStatus::Resolved,
        };

        let public_audit = PublicAudit {
            id: audit.id.to_hex(),
            auditor_id: audit.auditor_id.to_hex(),
            customer_id: audit.customer_id.to_hex(),
            project_id: audit.project_id.to_hex(),
            auditor_first_name: auditor.first_name,
            auditor_last_name: auditor.last_name,
            project_name: project.name,
            avatar: auditor.avatar,
            description: audit.description,
            status,
            scope: audit.scope,
            price: audit.price,
            auditor_contacts: auditor.contacts,
            customer_contacts: project.creator_contacts,
            tags: project.tags,
            last_modified: audit.last_modified,
            report: audit.report,
            report_name: audit.report_name,
            time: audit.time,
            issues: audit
                .issues
                .into_iter()
                .map(|i| auth.public_issue(i))
                .collect(),
            public: audit.public,
        };

        Ok(public_audit)
    }
}
