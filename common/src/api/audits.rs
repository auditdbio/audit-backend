use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    context::GeneralContext,
    entities::{
        audit::{Audit, AuditStatus, PublicAuditStatus},
        audit_request::TimeRange,
        auditor::{ExtendedAuditor, PublicAuditor},
        contacts::Contacts,
        issue::{Issue, Status},
        project::PublicProject,
    },
    error,
    services::{API_PREFIX, AUDITORS_SERVICE, CUSTOMERS_SERVICE, PROTOCOL},
};

use super::issue::PublicIssue;

#[derive(Debug, Serialize, Deserialize)]
pub enum AuditAction {
    #[serde(alias = "start")]
    Start,
    #[serde(alias = "resolve")]
    Resolve,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuditChange {
    pub avatar: Option<String>,
    pub action: Option<AuditAction>,
    pub project_name: Option<String>,
    pub description: Option<String>,
    pub scope: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub report_name: Option<String>,
    pub report: Option<String>,
    pub time: Option<TimeRange>,
    pub start_audit: Option<bool>,
    #[serde(rename = "isPublic")]
    pub public: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateIssue {
    pub name: String,
    pub description: String,
    pub status: Status,
    pub severity: String,
    pub category: String,
    #[serde(default)]
    pub links: Vec<String>,
    pub feedback: Option<String>,
}

impl CreateIssue {
    pub fn to_issue(self, id: usize) -> Issue<ObjectId> {
        Issue {
            id,
            name: self.name,
            description: self.description,
            status: self.status,
            severity: self.severity,
            events: Vec::new(),
            category: self.category,
            links: self.links,
            include: true,
            feedback: self.feedback.unwrap_or_default(),
            last_modified: Utc::now().timestamp(),
            read: HashMap::new(),
        }
    }

    pub fn to_issue_map(map: Vec<Self>) -> Vec<Issue<ObjectId>> {
        map.into_iter()
            .enumerate()
            .map(|(idx, issue)| issue.to_issue(idx))
            .collect()
    }
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

    #[serde(default)]
    pub no_customer: bool,
}

impl PublicAudit {
    pub async fn new(
        context: &GeneralContext,
        audit: Audit<ObjectId>,
    ) -> error::Result<PublicAudit> {
        let auth = context.auth();
        let auditor = context
            .make_request::<PublicAuditor>()
            .get(format!(
                "{}://{}/{}/auditor/{}",
                PROTOCOL.as_str(),
                AUDITORS_SERVICE.as_str(),
                API_PREFIX.as_str(),
                audit.auditor_id
            ))
            .auth(context.server_auth())
            .send()
            .await?
            .json::<ExtendedAuditor>()
            .await?;

        let project = match audit.no_customer {
            true => None,
            _ => Some(
                context
                    .make_request::<PublicProject>()
                    .get(format!(
                        "{}://{}/{}/project/{}",
                        PROTOCOL.as_str(),
                        CUSTOMERS_SERVICE.as_str(),
                        API_PREFIX.as_str(),
                        audit.project_id
                    ))
                    .auth(context.server_auth())
                    .send()
                    .await?
                    .json::<PublicProject>()
                    .await?,
            ),
        };

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

        let customer_contacts = if let Some(project) = &project {
            project.creator_contacts.clone()
        } else {
            Contacts {
                email: None,
                telegram: None,
                public_contacts: false,
            }
        };

        let project_name = if let Some(project) = &project {
            if audit.project_name == "" {
                project.name.clone()
            } else {
                audit.project_name
            }
        } else {
            audit.project_name
        };

        let public_audit = PublicAudit {
            id: audit.id.to_hex(),
            auditor_id: audit.auditor_id.to_hex(),
            customer_id: audit.customer_id.to_hex(),
            project_id: audit.project_id.to_hex(),
            auditor_first_name: auditor.first_name().clone(),
            auditor_last_name: auditor.last_name().clone(),
            project_name,
            avatar: auditor.avatar().clone(),
            description: audit.description,
            status,
            scope: audit.scope,
            price: audit.price,
            auditor_contacts: auditor.contacts().clone(),
            customer_contacts,
            tags: audit.tags,
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
            no_customer: audit.no_customer,
        };

        Ok(public_audit)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct NoCustomerAuditRequest {
    pub auditor_id: String,
    pub auditor_first_name: String,
    pub auditor_last_name: String,
    pub auditor_contacts: Contacts,
    pub avatar: String,

    pub project_name: String,
    pub description: String,
    pub status: AuditStatus,
    pub scope: Vec<String>,
    pub tags: Vec<String>,
    pub last_modified: i64,
    pub report: Option<String>,
    pub report_name: Option<String>,
    #[serde(rename = "isPublic")]
    pub public: bool,

    pub issues: Vec<CreateIssue>,
}
