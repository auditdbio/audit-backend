use std::hash::Hash;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    api::report::PublicReport,
    context::GeneralContext,
    repository::Entity,
    services::{API_PREFIX, PROTOCOL, REPORT_SERVICE},
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
    #[serde(rename = "Ready for resolve", alias = "Resolved")]
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
    pub price: i64,

    pub last_modified: i64,
    pub report: Option<String>,
    pub report_name: Option<String>,
    pub time: TimeRange,

    #[serde(default)]
    pub issues: Vec<Issue<Id>>,

    #[serde(default)]
    pub no_customer: bool,
}

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
            last_modified: self.last_modified,
            report: self.report,
            report_name: self.report_name,
            time: self.time,
            issues: Issue::parse_map(self.issues),
            public: self.public,
            no_customer: self.no_customer,
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
            last_modified: self.last_modified,
            report: self.report,
            report_name: self.report_name,
            time: self.time,
            issues: Issue::to_string_map(self.issues),
            public: self.public,
            no_customer: self.no_customer,
        }
    }

    pub async fn resolve(&mut self, context: &GeneralContext) {
        if self.report.is_none() {
            let public_report = context
                .make_request::<()>()
                .post(format!(
                    "{}://{}/{}/report/{}",
                    PROTOCOL.as_str(),
                    REPORT_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    self.id
                ))
                .auth(context.server_auth())
                .send()
                .await
                .unwrap()
                .json::<PublicReport>()
                .await;

            if let Err(err) = public_report {
                println!("Error in audit::resolve: {}", err);
                return;
            }
            let public_report = public_report.unwrap();
            self.report = Some(public_report.path.clone());
            self.report_name = Some(public_report.path);
        }
    }
}

impl Entity for Audit<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}
