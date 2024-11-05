use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::{distributions::Alphanumeric, Rng};

use crate::{
    api::{
        auditor::request_auditor,
        customer::request_customer,
        file::request_file_metadata,
    },
    context::GeneralContext,
    entities::{
        audit::{Audit, AuditStatus, PublicAuditStatus, ReportType},
        audit_request::TimeRange,
        contacts::Contacts,
        issue::{Issue, Status},
        project::PublicProject,
    },
    error,
    services::{API_PREFIX, CUSTOMERS_SERVICE, PROTOCOL},
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
    pub report_type: Option<ReportType>,
    pub report: Option<String>,
    pub time: Option<TimeRange>,
    pub start_audit: Option<bool>,
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
            edit_history: Vec::new(),
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

    pub auditor_first_name: String,
    pub auditor_last_name: String,
    pub avatar: String,
    pub auditor_contacts: Contacts,

    pub customer_first_name: String,
    pub customer_last_name: String,
    pub customer_avatar: String,
    pub customer_contacts: Contacts,

    pub project_name: String,
    pub description: String,
    pub scope: Vec<String>,
    pub tags: Vec<String>,
    pub status: PublicAuditStatus,
    pub price: Option<i64>,
    pub total_cost: Option<i64>,

    pub time: TimeRange,
    pub last_modified: i64,
    pub resolved_at: Option<i64>,
    pub report: Option<String>,
    pub report_name: Option<String>,
    #[serde(rename = "isPublic")]
    pub public: bool,

    pub issues: Vec<PublicIssue>,

    #[serde(default)]
    pub no_customer: bool,
    pub conclusion: Option<String>,
    pub access_code: Option<String>,
    pub report_sha: Option<String>,
}

impl PublicAudit {
    pub async fn new(
        context: &GeneralContext,
        audit: Audit<ObjectId>,
        only_public: bool,
    ) -> error::Result<PublicAudit> {
        let auth = context.auth();

        let auditor = request_auditor(&context, audit.auditor_id, context.server_auth()).await?;
        let customer = match audit.no_customer {
            true => None,
            _ => Some(request_customer(&context, audit.customer_id, context.server_auth()).await?),
        };

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

        let mut auditor_contacts = auditor.contacts().clone();

        if !auditor_contacts.public_contacts && only_public {
            auditor_contacts = Contacts {
                email: None,
                telegram: None,
                public_contacts: false,
            }
        }

        let customer_contacts = customer
            .clone()
            .map(|customer| {
                if !customer.contacts.public_contacts && only_public {
                    Contacts {
                        email: None,
                        telegram: None,
                        public_contacts: false,
                    }
                } else {
                    customer.contacts
                }
            })
            .unwrap_or_else(|| Contacts {
                email: None,
                telegram: None,
                public_contacts: false,
            });

        let (customer_first_name, customer_last_name, customer_avatar) = if let Some(customer) = customer {
            (customer.first_name, customer.last_name, customer.avatar)
        } else {
            ("".to_string(), "".to_string(), "".to_string())
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

        let report_name = if let Some(report) = audit.report.clone() {
            let meta = request_file_metadata(&context, report, context.server_auth()).await?;
            if let Some(meta) = meta {
                Some(format!(
                    "{}.{}",
                    meta.original_name.unwrap_or("Report".to_string()),
                    meta.extension,
                ))
            } else {
                None
            }
        } else {
            None
        };

        let public_audit = PublicAudit {
            id: audit.id.to_hex(),
            auditor_id: audit.auditor_id.to_hex(),
            customer_id: audit.customer_id.to_hex(),
            project_id: audit.project_id.to_hex(),
            auditor_first_name: auditor.first_name().clone(),
            auditor_last_name: auditor.last_name().clone(),
            avatar: auditor.avatar().clone(),
            auditor_contacts,
            customer_first_name,
            customer_last_name,
            customer_avatar,
            customer_contacts,
            project_name,
            description: audit.description,
            tags: audit.tags,
            scope: audit.scope,
            status,
            price,
            total_cost,
            time: audit.time,
            last_modified: audit.last_modified,
            resolved_at: audit.resolved_at,
            report: audit.report,
            report_name,
            issues,
            public: audit.public,
            no_customer: audit.no_customer,
            conclusion: audit.conclusion,
            access_code,
            report_sha: audit.report_sha,
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
    pub scope: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
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
