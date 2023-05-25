use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{
        audit::Audit,
        audit_request::{AuditRequest, TimeRange},
        auditor::PublicAuditor,
        contacts::Contacts,
        issue::{ChangeIssue, Issue, Status, Event},
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
    pub status: String,
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
    pub link: String,
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
            status: "pending".to_string(),
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
                    .find_many("auditor_id", &Bson::ObjectId(auth.id().unwrap().clone()))
                    .await?
            }
            Role::Customer => {
                audits
                    .find_many("customer_id", &Bson::ObjectId(auth.id().unwrap().clone()))
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

        if let Some(status) = change.status {
            audit.status = status;
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
            link: issue.link,
            include: true,
            feedback: String::new(),
            last_modified: Utc::now().timestamp(),
        };

        audit.issues.push(issue.clone());

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        audits.delete("id", &audit_id).await?;

        audits.insert(&audit).await?;

        Ok(issue.to_string())
    }

    pub async fn change_issue(
        &self,
        audit_id: ObjectId,
        issue_id: usize,
        change: ChangeIssue,
    ) -> error::Result<Issue<String>> {
        let Some(mut audit) = self.get_audit(audit_id).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        if !change.get_access(&audit, self.context.auth()) {
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

        if let Some(action) = change.status {
            let Some(new_state) = issue.status.apply(&action) else {
                return Err(anyhow::anyhow!("Invalid action").code(400));
            };
            issue.status = new_state;
        }

        if let Some(severity) = change.severity {
            issue.severity = severity;
        }

        if let Some(category) = change.category {
            issue.category = category;
        }

        if let Some(link) = change.link {
            issue.link = link;
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
                    user: self.context.auth().id().unwrap().clone(),
                    kind: create_event.kind,
                    message: create_event.message,
                    id: issue.events.len(),
                };

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
}
