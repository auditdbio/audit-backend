use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::{distributions::Alphanumeric, Rng};

use crate::{
    context::GeneralContext,
    entities::{
        audit::{Audit, AuditStatus, PublicAuditStatus, ReportType},
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
    pub price: Option<i64>,
    pub total_cost: Option<i64>,
    pub report_name: Option<String>,
    pub report_type: Option<ReportType>,
    pub report: Option<String>,
    pub time: Option<TimeRange>,
    pub start_audit: Option<bool>,
    #[serde(rename = "isPublic")]
    pub public: Option<bool>,
    pub conclusion: Option<String>,
    pub comment: Option<String>,
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
            last_modified: Utc::now().timestamp_micros(),
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
    pub price: Option<i64>,
    pub total_cost: Option<i64>,

    pub auditor_contacts: Contacts,
    pub customer_contacts: Contacts,
    pub tags: Vec<String>,
    pub last_modified: i64,
    pub resolved_at: Option<i64>,
    pub report: Option<String>,
    pub report_name: Option<String>,
    pub time: TimeRange,

    pub issues: Vec<PublicIssue>,

    #[serde(default)]
    pub no_customer: bool,
    pub conclusion: Option<String>,
    pub access_code: Option<String>,
}

impl PublicAudit {
    pub async fn new(
        context: &GeneralContext,
        audit: Audit<ObjectId>,
        only_public: bool,
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

        let is_audit_approved = if audit.edit_history.is_empty() || audit.approved_by.is_empty() {
            true
        } else {
            let first = audit.approved_by.values().next().unwrap();
            audit.approved_by.values().all(|v| v == first)
        };

        let status = match audit.status {
            AuditStatus::Waiting => PublicAuditStatus::WaitingForAudit,
            AuditStatus::Started => {
                // else if audit.report.is_some() {
                //     PublicAuditStatus::ReadyForResolve
                if !is_audit_approved {
                    PublicAuditStatus::ApprovalNeeded
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

        let (price, total_cost, access_code) = if only_public {
            (None, None, None)
        } else {
            (audit.price, audit.total_cost, audit.access_code)
        };

        let mut issues = audit
            .issues
            .into_iter()
            .map(|i| {
                let mut public_issue = auth.public_issue(i);
                if only_public {
                    public_issue.events = vec![];
                }
                public_issue
            })
            .collect::<Vec<PublicIssue>>();

        if only_public {
            issues.retain(|issue| issue.include)
        }

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
            price,
            total_cost,
            auditor_contacts: auditor.contacts().clone(),
            customer_contacts,
            tags: audit.tags,
            last_modified: audit.last_modified,
            resolved_at: audit.resolved_at,
            report: audit.report,
            report_name: audit.report_name,
            time: audit.time,
            issues,
            public: audit.public,
            no_customer: audit.no_customer,
            conclusion: audit.conclusion,
            access_code,
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
    pub conclusion: Option<String>,
}

pub fn create_access_code() -> String {
    let time = Utc::now().timestamp_micros().to_string();
    let rnd: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    format!("{}{}", time, rnd)
}
