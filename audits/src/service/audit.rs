use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{send_notification, NewNotification},
    context::Context,
    entities::{
        audit::{Audit, AuditStatus},
        audit_request::{AuditRequest, TimeRange},
        auditor::PublicAuditor,
        contacts::Contacts,
        issue::{ChangeIssue, Event, EventKind, Issue, Status},
        project::PublicProject,
        role::Role,
    },
    error::{self, AddCode},
    services::{CUSTOMERS_SERVICE, PROTOCOL},
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

use super::audit_request::PublicRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditChange {
    pub avatar: Option<String>,
    pub status: Option<String>,
    pub scope: Option<Vec<String>>,
    pub report_name: Option<String>,
    pub report: Option<String>,
    pub time: Option<TimeRange>,
    pub start_audit: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicAudit {
    pub id: String,
    pub auditor_id: String,
    pub customer_id: String,
    pub project_id: String,

    pub auditor_first_name: String,
    pub auditor_last_name: String,

    pub project_name: String,
    pub avatar: String,
    pub description: String,
    pub status: AuditStatus,
    pub scope: Vec<String>,
    pub price: i64,

    pub auditor_contacts: Contacts,
    pub customer_contacts: Contacts,
    pub tags: Vec<String>,
    pub last_modified: i64,
    pub report: Option<String>,
    pub report_name: Option<String>,
    pub time: TimeRange,

    pub issues: Vec<Issue<String>>,
}

impl PublicAudit {
    pub async fn new(context: &Context, audit: Audit<ObjectId>) -> error::Result<PublicAudit> {
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
            status: audit.status,
            scope: audit.scope,
            price: audit.price,
            auditor_contacts: auditor.contacts,
            customer_contacts: project.creator_contacts,
            tags: project.tags,
            last_modified: audit.last_modified,
            report: audit.report,
            report_name: audit.report_name,
            time: audit.time,
            issues: Issue::to_string_map(audit.issues),
        };

        Ok(public_audit)
    }
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

pub struct AuditService {
    context: Context,
}

impl AuditService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, request: PublicRequest) -> error::Result<PublicAudit> {
        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let audit = Audit {
            id: ObjectId::new(),
            customer_id: request.customer_id.parse()?,
            auditor_id: request.auditor_id.parse()?,
            project_id: request.project_id.parse()?,
            description: request.description,
            status: AuditStatus::WaitingForAudit,
            scope: request.project_scope,
            price: request.price,
            last_modified: Utc::now().timestamp_micros(),
            report: None,
            report_name: None,
            time: request.time,
            issues: Vec::new(),
        };

        audits.insert(&audit).await?;

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        requests.delete("id", &request.id.parse()?).await?;

        let public_audit = PublicAudit::new(&self.context, audit).await?;

        Ok(public_audit)
    }

    async fn get_audit(&self, id: ObjectId) -> error::Result<Option<Audit<ObjectId>>> {
        let auth = self.context.auth();

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let Some(audit) = audits.find("id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(auth, &audit) {
            return Err(anyhow::anyhow!("User is not available to change this audit").code(403));
        }

        Ok(Some(audit))
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicAudit>> {
        let audit = self.get_audit(id).await?;

        if let Some(audit) = audit {
            let public_audit = PublicAudit::new(&self.context, audit).await?;

            return Ok(Some(public_audit));
        }

        Ok(None)
    }

    pub async fn my_audit(&self, role: Role) -> error::Result<Vec<PublicAudit>> {
        let auth = self.context.auth();

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let audits = match role {
            Role::Auditor => {
                audits
                    .find_many("auditor_id", &Bson::ObjectId(*auth.id().unwrap()))
                    .await?
            }
            Role::Customer => {
                audits
                    .find_many("customer_id", &Bson::ObjectId(*auth.id().unwrap()))
                    .await?
            }
        };

        let mut public_audits = Vec::new();

        for audit in audits {
            public_audits.push(PublicAudit::new(&self.context, audit).await?);
        }

        Ok(public_audits)
    }

    pub async fn change(&self, id: ObjectId, change: AuditChange) -> error::Result<PublicAudit> {
        let auth = self.context.auth();

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let Some(mut audit) = audits.find("id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        if !Edit.get_access(auth, &audit) {
            return Err(anyhow::anyhow!("User is not available to change this audit").code(403));
        }

        if let Some(scope) = change.scope {
            audit.scope = scope;
        }

        if let Some(report) = change.report {
            audit.report = Some(report);
        }

        if let Some(report_name) = change.report_name {
            audit.report_name = Some(report_name);
        }

        if let Some(start_audit) = change.start_audit {
            if start_audit {
                audit.status = AuditStatus::InProgress;
            }
        }

        if audit.status != AuditStatus::WaitingForAudit {
            if audit.report.is_some() {
                audit.status = AuditStatus::Resolved;
            } else if audit.issues.is_empty() {
                audit.status = AuditStatus::InProgress;
            } else if audit.issues.iter().all(|issue| issue.is_resolved()) {
                audit.status = AuditStatus::Resolved;
            } else {
                audit.status = AuditStatus::IssuesWorkflow;
            }
        }

        audit.last_modified = Utc::now().timestamp_micros();

        audits.delete("id", &id).await?;
        audits.insert(&audit).await?;

        let public_audit = PublicAudit::new(&self.context, audit).await?;

        Ok(public_audit)
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<PublicAudit> {
        let auth = self.context.auth();

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let Some(audit) = audits.delete("id", &id).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        if !Edit.get_access(auth, &audit) {
            audits.insert(&audit).await?;
            return Err(anyhow::anyhow!("User is not available to delete this audit").code(403));
        }

        let public_audit = PublicAudit::new(&self.context, audit).await?;

        Ok(public_audit)
    }

    pub async fn create_issue(
        &self,
        audit_id: ObjectId,
        issue: CreateIssue,
    ) -> error::Result<Issue<String>> {
        let Some(mut audit) = self.get_audit(audit_id).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        let issue: Issue<ObjectId> = Issue {
            id: audit.issues.len(),
            name: issue.name,
            description: issue.description,
            status: issue.status,
            severity: issue.severity,
            events: Vec::new(),
            category: issue.category,
            links: issue.links,
            include: true,
            feedback: String::new(),
            last_modified: Utc::now().timestamp(),
        };

        audit.issues.push(issue.clone());

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        audits.delete("id", &audit_id).await?;

        audits.insert(&audit).await?;

        let mut new_notification: NewNotification =
            serde_json::from_str(include_str!("../../templates/audit_issue_disclosed.txt"))?;

        new_notification.user_id = Some(audit.customer_id);

        send_notification(&self.context, true, true, new_notification).await?;

        Ok(issue.to_string())
    }

    pub async fn change_issue(
        &self,
        audit_id: ObjectId,
        issue_id: usize,
        change: ChangeIssue,
    ) -> error::Result<Issue<String>> {
        let auth = self.context.auth();
        let Some(mut audit) = self.get_audit(audit_id).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        if !change.get_access(&audit, auth) {
            return Err(anyhow::anyhow!("User is not available to change this issue").code(403));
        }

        let Some(mut issue) = audit.issues.get(issue_id).cloned() else {
            return Err(anyhow::anyhow!("No issue found").code(404));
        };

        if let Some(name) = change.name {
            issue.name = name;
        }

        if let Some(description) = change.description {
            issue.description = description;
        }

        let receiver_id = if auth.id().unwrap() == &audit.customer_id {
            audit.auditor_id
        } else {
            audit.customer_id
        };

        if let Some(action) = change.status {
            let Some(new_state) = issue.status.apply(&action) else {
                return Err(anyhow::anyhow!("Invalid action").code(400));
            };
            let mut new_notification: NewNotification = serde_json::from_str(include_str!(
                "../../templates/audit_issue_status_change.txt"
            ))?;

            new_notification.user_id = Some(receiver_id);

            send_notification(&self.context, true, true, new_notification).await?;
            issue.status = new_state;
        }

        if let Some(severity) = change.severity {
            issue.severity = severity;
        }

        if let Some(category) = change.category {
            issue.category = category;
        }

        if let Some(links) = change.links {
            issue.links = links;
        }

        if let Some(include) = change.include {
            issue.include = include;
        }

        if let Some(feedback) = change.feedback {
            issue.feedback = feedback;
        }

        if let Some(events) = change.events {
            for create_event in events {
                let event = Event {
                    timestamp: Utc::now().timestamp(),
                    user: *self.context.auth().id().unwrap(),
                    kind: create_event.kind,
                    message: create_event.message,
                    id: issue.events.len(),
                };

                if event.kind == EventKind::Comment {
                    let mut new_notification: NewNotification = serde_json::from_str(
                        include_str!("../../templates/audit_issue_comment.txt"),
                    )?;

                    new_notification.user_id = Some(receiver_id);

                    send_notification(&self.context, true, true, new_notification).await?;
                }

                issue.events.push(event);
            }
        }

        issue.last_modified = Utc::now().timestamp();

        audit.issues[issue_id] = issue.clone();

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        audits.delete("id", &audit_id).await?;

        audits.insert(&audit).await?;

        Ok(issue.to_string())
    }

    pub async fn get_issues(&self, audit_id: ObjectId) -> error::Result<Vec<Issue<String>>> {
        let audit = self.get_audit(audit_id).await?;

        if let Some(audit) = audit {
            let issues = audit.issues;

            let issues: Vec<Issue<String>> = issues
                .into_iter()
                .map(|issue| issue.to_string())
                .collect::<Vec<Issue<String>>>();

            return Ok(issues);
        }

        Ok(Vec::new())
    }

    pub async fn get_issue_by_id(
        &self,
        audit_id: ObjectId,
        issue_id: usize,
    ) -> error::Result<Issue<String>> {
        let audit = self.get_audit(audit_id).await?;

        if let Some(audit) = audit {
            let issue = audit.issues.get(issue_id).cloned();

            if let Some(issue) = issue {
                return Ok(issue.to_string());
            }
        }

        Err(anyhow::anyhow!("No issue found").code(404))
    }
}
