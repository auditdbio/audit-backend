use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{
        audits::{AuditAction, AuditChange, CreateIssue, PublicAudit},
        send_notification, NewNotification,
    },
    context::Context,
    entities::{
        audit::{Audit, AuditStatus},
        audit_request::AuditRequest,
        issue::{ChangeIssue, Event, EventKind, Issue},
        role::Role,
    },
    error::{self, AddCode},
};
use mongodb::bson::{oid::ObjectId, Bson};

use super::audit_request::PublicRequest;

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
            status: AuditStatus::Waiting,
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

        if audit.status != AuditStatus::Resolved {
            if let Some(scope) = change.scope {
                audit.scope = scope;
            }

            if let Some(report) = change.report {
                audit.report = Some(report);
            }

            if let Some(report_name) = change.report_name {
                audit.report_name = Some(report_name);
            }
        }

        if let Some(action) = change.action {
            match action {
                AuditAction::Start => {
                    if audit.status == AuditStatus::Waiting {
                        audit.status = AuditStatus::Started;
                    }
                }
                AuditAction::Resolve => {
                    if audit.status == AuditStatus::Started {
                        audit.status = AuditStatus::Resolved;
                    }
                }
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
                .map(Issue::to_string)
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
