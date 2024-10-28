use std::{collections::HashMap, hash::Hash};
use chrono::Utc;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    impl_has_last_modified,
    api::{
        audits::create_access_code,
        chat::AuditMessageId,
        report::{PublicReport, CreateReport},
    },
    entities::{auditor::ExtendedAuditor, customer::PublicCustomer, role::Role},
    error::{self, AddCode},
    context::GeneralContext,
    repository::{Entity, HasLastModified},
    services::{API_PREFIX, PROTOCOL, AUDITORS_SERVICE, CUSTOMERS_SERVICE, REPORT_SERVICE},
};

use super::{audit_request::TimeRange, issue::Issue};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PublicAuditStatus {
    #[serde(rename = "Waiting for audit", alias = "WaitingForAudit")]
    WaitingForAudit,
    #[serde(rename = "In progress", alias = "InProgress")]
    InProgress,
    #[serde(rename = "Issues workflow", alias = "IssuesWorkflow")]
    IssuesWorkflow,
    #[serde(rename = "Approval needed", alias = "ApprovalNeeded")]
    ApprovalNeeded,
    #[serde(rename = "Ready for resolve", alias = "ReadyForResolve")]
    ReadyForResolve,
    #[serde(rename = "Resolved", alias = "Resolved")]
    Resolved,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AuditStatus {
    Waiting,
    Started,
    Resolved,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ReportType {
    Generated,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Audit<Id: Eq + Hash> {
    #[serde(rename = "_id")]
    pub id: Id,
    pub customer_id: Id,
    pub auditor_id: Id,
    pub project_id: Id,
    #[serde(default)]
    pub public: bool,

    #[serde(default)]
    pub project_name: String,
    pub description: String,
    pub status: AuditStatus,
    pub scope: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub price: Option<i64>,
    pub total_cost: Option<i64>,

    pub last_modified: i64,
    pub resolved_at: Option<i64>,
    pub report: Option<String>,
    pub report_name: Option<String>,
    pub report_type: Option<ReportType>,
    pub time: TimeRange,

    #[serde(default)]
    pub issues: Vec<Issue<Id>>,

    #[serde(default)]
    pub edit_history: Vec<AuditEditHistory>,
    #[serde(default)]
    pub approved_by: HashMap<String, usize>,
    #[serde(default)]
    pub unread_edits: HashMap<String, usize>,

    #[serde(default)]
    pub no_customer: bool,
    pub chat_id: Option<AuditMessageId>,
    pub conclusion: Option<String>,

    pub access_code: Option<String>,
    pub report_sha: Option<String>,
}

impl_has_last_modified!(Audit<ObjectId>);

impl Audit<String> {
    pub fn parse(self) -> Audit<ObjectId> {
        Audit {
            id: self.id.parse().unwrap(),
            customer_id: self.customer_id.parse().unwrap(),
            auditor_id: self.auditor_id.parse().unwrap(),
            project_id: self.project_id.parse().unwrap(),
            project_name: self.project_name,
            description: self.description,
            status: self.status,
            scope: self.scope,
            tags: self.tags,
            price: self.price,
            total_cost: self.total_cost,
            last_modified: self.last_modified,
            resolved_at: self.resolved_at,
            report: self.report,
            report_name: self.report_name,
            report_type: self.report_type,
            time: self.time,
            issues: Issue::parse_map(self.issues),
            public: self.public,
            no_customer: self.no_customer,
            chat_id: self.chat_id,
            conclusion: self.conclusion,
            edit_history: self.edit_history,
            approved_by: self.approved_by,
            unread_edits: self.unread_edits,
            access_code: self.access_code,
            report_sha: self.report_sha,
        }
    }
}

impl Audit<ObjectId> {
    pub fn stringify(self) -> Audit<String> {
        Audit {
            id: self.id.to_hex(),
            customer_id: self.customer_id.to_hex(),
            auditor_id: self.auditor_id.to_hex(),
            project_id: self.project_id.to_hex(),
            project_name: self.project_name,
            description: self.description,
            status: self.status,
            scope: self.scope,
            tags: self.tags,
            price: self.price,
            total_cost: self.total_cost,
            last_modified: self.last_modified,
            resolved_at: self.resolved_at,
            report: self.report,
            report_name: self.report_name,
            report_type: self.report_type,
            time: self.time,
            issues: Issue::to_string_map(self.issues),
            public: self.public,
            no_customer: self.no_customer,
            chat_id: self.chat_id,
            conclusion: self.conclusion,
            edit_history: self.edit_history,
            approved_by: self.approved_by,
            unread_edits: self.unread_edits,
            access_code: self.access_code,
            report_sha: self.report_sha,
        }
    }

    pub async fn resolve(&mut self, context: &GeneralContext) -> error::Result<()> {
        self.resolved_at = Some(Utc::now().timestamp_micros());
        let access_code = create_access_code();
        self.access_code = Some(access_code.clone());

        if self.report_type.is_none() || self.report_type.clone().unwrap() == ReportType::Generated {
            let report_response = context
                .make_request()
                .post(format!(
                    "{}://{}/{}/report/{}?code={}",
                    PROTOCOL.as_str(),
                    REPORT_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    self.id,
                    access_code,
                ))
                .auth(context.server_auth())
                .json(&CreateReport {
                    is_draft: Some(false),
                })
                .send()
                .await;

            if let Err(e) = report_response {
                return Err(anyhow::anyhow!(format!("Error in report request: {}", e)).code(502));
            }

            let report_response = report_response.unwrap();
            if report_response.status().is_success() {
                let public_report = report_response.json::<PublicReport>().await;
                if let Err(e) = public_report {
                    return Err(anyhow::anyhow!(format!("Error in report response json: {}", e)).code(404));
                }
                let public_report = public_report.unwrap();
                self.report = Some(public_report.path.clone());
                self.report_name = Some(public_report.path);
                self.report_type = Some(ReportType::Generated);
                self.report_sha = public_report.report_sha;
            } else {
                return Err(
                    anyhow::anyhow!(
                        format!("Report receiving error: {}", report_response.status())
                    ).code(502)
                );
            }
        }

        Ok(())
    }
}

impl Entity for Audit<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AuditEditHistory {
    pub id: usize,
    pub date: i64,
    pub author: String,
    pub comment: Option<String>,
    pub audit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EditHistoryAuthor {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub avatar: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicAuditEditHistory {
    pub id: usize,
    pub date: i64,
    pub author: EditHistoryAuthor,
    pub comment: Option<String>,
    pub audit: String,
}

impl PublicAuditEditHistory {
    pub async fn new(
        context: &GeneralContext,
        history: AuditEditHistory,
        role: Role,
    ) -> error::Result<PublicAuditEditHistory> {
        let author = if role == Role::Auditor {
            let auditor = context
                .make_request::<ExtendedAuditor>()
                .get(format!(
                    "{}://{}/{}/auditor/{}",
                    PROTOCOL.as_str(),
                    AUDITORS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    history.author,
                ))
                .auth(context.server_auth())
                .send()
                .await?
                .json::<ExtendedAuditor>()
                .await?;

            EditHistoryAuthor {
                id: history.author,
                name: format!("{} {}", auditor.first_name(), auditor.last_name()),
                role,
                avatar: auditor.avatar().clone(),
            }
        } else {
            let customer = context
                .make_request::<PublicCustomer>()
                .get(format!(
                    "{}://{}/{}/customer/{}",
                    PROTOCOL.as_str(),
                    CUSTOMERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    history.author,
                ))
                .auth(context.server_auth())
                .send()
                .await?
                .json::<PublicCustomer>()
                .await?;

            EditHistoryAuthor {
                id: history.author,
                name: format!("{} {}", customer.first_name, customer.last_name),
                role,
                avatar: customer.avatar,
            }
        };

        Ok(PublicAuditEditHistory {
            id: history.id,
            date: history.date,
            author,
            comment: history.comment,
            audit: history.audit,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EditHistoryResponse {
    pub edit_history: Vec<PublicAuditEditHistory>,
    pub approved_by: HashMap<String, usize>,
    pub unread: HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ChangeAuditHistory {
    pub comment: Option<String>,
    pub is_approved: Option<bool>,
}
