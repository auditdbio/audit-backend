use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::Utc;
use rand::Rng;
use mongodb::bson::{oid::ObjectId, Bson};

use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{
        audits::{AuditAction, AuditChange, CreateIssue, PublicAudit, NoCustomerAuditRequest},
        chat::{
            send_message, create_audit_message,
            delete_message, AuditMessageStatus,
            CreateAuditMessage, AuditMessageId,
        },
        events::{post_event, EventPayload, PublicEvent},
        issue::PublicIssue,
        send_notification, NewNotification,
        seartch::PaginationParams,
    },
    context::GeneralContext,
    entities::{
        audit::{
            Audit, AuditStatus,
            AuditEditHistory, PublicAuditEditHistory,
            ChangeAuditHistory, EditHistoryResponse,
        },
        audit_request::{AuditRequest, TimeRange},
        issue::{severity_to_integer, ChangeIssue, Event, EventKind, Issue, Status, Action},
        project::get_project,
        role::Role,
    },
    error::{self, AddCode},
};

use super::audit_request::PublicRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct MyAuditResult {
    pub result: Vec<PublicAudit>,
    #[serde(rename = "totalDocuments")]
    pub total_documents: u64,
}

pub struct AuditService {
    context: GeneralContext,
}

impl AuditService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn create(&self, request: PublicRequest) -> error::Result<PublicAudit> {
        let auth = self.context.auth();
        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let user_id = auth.id().unwrap();
        let auditor_id: ObjectId = request.auditor_id.parse()?;
        let customer_id: ObjectId = request.customer_id.parse()?;

        let tags = if let Some(request_tags) = request.tags {
            request_tags
        } else {
            Vec::new()
        };

        let mut audit = Audit {
            id: request.id.parse()?,
            customer_id,
            auditor_id,
            project_id: request.project_id.parse()?,
            project_name: request.project_name,
            description: request.description,
            status: AuditStatus::Waiting,
            scope: request.project_scope,
            tags,
            price: request.price,
            total_cost: request.total_cost,
            last_modified: Utc::now().timestamp_micros(),
            report: None,
            report_name: None,
            time: request.time,
            issues: Vec::new(),
            public: false,
            no_customer: false,
            chat_id: None,
            conclusion: None,
            edit_history: Vec::new(),
            approved_by: HashMap::new(),
        };

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        if let Some(request) = requests
            .delete("id", &request.id.parse()?)
            .await? {
            let edit_history = request.edit_history;
            audit.edit_history = edit_history.clone();
            if let Some(last) = edit_history.last() {
                audit.approved_by.insert(audit.customer_id.to_hex(), last.id);
                audit.approved_by.insert(audit.auditor_id.to_hex(), last.id);
            }

            if let Some(chat_id) = request.chat_id {
                delete_message(chat_id.chat_id, chat_id.message_id, auth.clone())?
            }
        }

        let public_audit = PublicAudit::new(&self.context, audit.clone()).await?;

        let (event_receiver, receiver_role, last_changer) = if user_id == customer_id {
            (auditor_id, Role::Auditor, Role::Customer)
        } else {
            (customer_id, Role::Customer, Role::Auditor)
        };

        let message = create_audit_message(
            CreateAuditMessage::Audit(public_audit.clone()),
            Some(audit.status.clone().into()),
            event_receiver,
            receiver_role,
            last_changer
        );

        let chat = send_message(message, auth)?;

        audit.chat_id = Some(AuditMessageId {
            chat_id: chat.id,
            message_id: chat.last_message.id,
        });

        audits.insert(&audit).await?;

        let event = PublicEvent::new(event_receiver, EventPayload::NewAudit(public_audit.clone()));

        post_event(&self.context, event, self.context.server_auth()).await?;

        let event = PublicEvent::new(
            event_receiver,
            EventPayload::RequestAccept(request.id.clone()),
        );

        post_event(&self.context, event, self.context.server_auth()).await?;

        Ok(public_audit)
    }

    pub async fn create_no_customer(
        &self,
        request: NoCustomerAuditRequest,
    ) -> error::Result<PublicAudit> {
        let auth = self.context.auth();
        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let auditor_id: ObjectId = request.auditor_id.parse()?;
        let customer_id: ObjectId = auditor_id;

        let time = TimeRange {
            from: Utc::now().timestamp_micros(),
            to: Utc::now().timestamp_micros(),
        };

        let audit = Audit {
            id: ObjectId::new(),
            customer_id,
            auditor_id,
            project_id: ObjectId::new(),
            project_name: request.project_name,
            description: request.description,
            status: request.status,
            scope: request.scope,
            tags: request.tags,
            price: None,
            total_cost: None,
            last_modified: Utc::now().timestamp_micros(),
            report: None,
            report_name: None,
            time,
            public: false,
            no_customer: true,
            issues: CreateIssue::to_issue_map(request.issues),
            chat_id: None,
            conclusion: request.conclusion,
            edit_history: Vec::new(),
            approved_by: HashMap::new(),
        };

        if !Edit.get_access(&auth, &audit) {
            return Err(anyhow::anyhow!("User is not available to create this audit").code(403));
        }

        audits.insert(&audit).await?;

        Ok(PublicAudit::new(&self.context, audit).await?)
    }

    async fn get_audit(&self, id: ObjectId) -> error::Result<Option<Audit<ObjectId>>> {
        let auth = self.context.auth();

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let Some(audit) = audits.find("_id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(&auth, &audit) {
            return Err(anyhow::anyhow!("User is not available to read this audit").code(403));
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

    pub async fn my_audit(
        &self,
        role: Role,
        pagination: PaginationParams
    ) -> error::Result<Vec<PublicAudit>> {
        let page = pagination.page.unwrap_or(0);
        let per_page = pagination.per_page.unwrap_or(0);
        let limit = pagination.per_page.unwrap_or(1000);
        let skip = (page - 1) * per_page;

        let auth = self.context.auth();

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let (audits, _total_documents) = match role {
            Role::Auditor => {
                audits
                    .find_many_limit(
                        "auditor_id",
                        &Bson::ObjectId(auth.id().unwrap()),
                        skip,
                        limit,
                    ).await?
            }
            Role::Customer => {
                audits
                    .find_many_limit(
                        "customer_id",
                        &Bson::ObjectId(auth.id().unwrap()),
                        skip,
                        limit,
                    ).await?
            }
        };

        let mut public_audits = Vec::new();

        for audit in audits {
            public_audits.push(PublicAudit::new(&self.context, audit).await?);
        }

        // Ok(MyAuditResult {
        //     result: public_audits,
        //     total_documents,
        // })
        Ok(public_audits)
    }

    pub async fn change(&self, id: ObjectId, change: AuditChange) -> error::Result<PublicAudit> {
        let auth = self.context.auth();
        let user_id = auth.id().unwrap();

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let Some(mut audit) = audits.find("_id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        if !Edit.get_access(&auth, &audit) {
            return Err(anyhow::anyhow!("User is not available to change this audit").code(403));
        }

        let mut is_history_changed = false;

        if let Some(public) = change.public {
            audit.public = public;
        }

        if audit.status != AuditStatus::Resolved || audit.no_customer {
            if let Some(scope) = change.scope.clone() {
                if audit.scope != scope {
                    audit.scope = scope;
                    is_history_changed = true;
                }
            }
            if let Some(description) = change.description {
                if audit.description != description {
                    audit.description = description;
                    is_history_changed = true;
                }
            }
            if let Some(tags) = change.tags {
                if audit.tags != tags {
                    audit.tags = tags;
                    is_history_changed = true;
                }
            }
            if change.total_cost.is_some() {
                if audit.total_cost != change.total_cost {
                    audit.total_cost = change.total_cost;
                    is_history_changed = true;
                }
            }
            if change.price.is_some() && change.total_cost.is_none() {
                if audit.price != change.price {
                    audit.price = change.price;
                    is_history_changed = true;
                }
            }
            if let Some(conclusion) = change.conclusion {
                if user_id == audit.auditor_id {
                    if audit.conclusion.is_some() && audit.conclusion.clone().unwrap() != conclusion {
                        audit.conclusion = Some(conclusion);
                        is_history_changed = true;
                    }
                }
            }
        }

        if audit.no_customer {
            if let Some(project_name) = change.project_name {
                if audit.project_name != project_name {
                    audit.project_name = project_name;
                    is_history_changed = true;
                }
            }
        }

        if let Some(ref report) = change.report {
            audit.report = Some(report.clone());
        }

        if let Some(ref report_name) = change.report_name {
            audit.report_name = Some(report_name.clone());
        }

        let is_audit_approved = if audit.edit_history.is_empty() || audit.approved_by.is_empty() {
            true
        } else {
            let first = audit.approved_by.values().next().unwrap();
            audit.approved_by.values().all(|v| v == first)
        };

        if let Some(ref action) = change.action {
            match action {
                AuditAction::Start => {
                    if audit.status == AuditStatus::Waiting {
                        audit.status = AuditStatus::Started;
                    }
                }
                AuditAction::Resolve => {
                    if !is_audit_approved {
                        return Err(anyhow::anyhow!("Audit approval is required from all participants").code(404));
                    } else if audit.status == AuditStatus::Started {
                        audit.status = AuditStatus::Resolved;
                        audit.resolve(&self.context).await?;
                    }
                }
            }
        }

        audit.last_modified = Utc::now().timestamp_micros();

        let (
            event_receiver,
            receiver_role,
            last_changer_role,
        ) = if user_id == audit.customer_id {
            (audit.auditor_id, Role::Auditor, Role::Customer)
        } else {
            (audit.customer_id, Role::Customer, Role::Auditor)
        };

        if is_history_changed {
            let edit_history_item = AuditEditHistory {
                id: audit.edit_history.len(),
                date: audit.last_modified.clone(),
                author: user_id.to_hex(),
                comment: change.comment,
                audit: serde_json::to_string(&json!({
                    "project_name": audit.project_name,
                    "description": audit.description,
                    "scope": audit.scope,
                    "tags": audit.tags,
                    "price": audit.price,
                    "total_cost": audit.total_cost,
                    "time": audit.time,
                    "conclusion": audit.conclusion,
                })).unwrap(),
            };

            audit.edit_history.push(edit_history_item.clone());

            let is_approved = is_audit_approved
                && change.price.is_none()
                && change.total_cost.is_none()
                && change.scope.is_none();

            if is_approved {
                audit.approved_by.insert(audit.auditor_id.to_hex(), edit_history_item.id.clone());
                audit.approved_by.insert(audit.customer_id.to_hex(), edit_history_item.id);
            } else {
                audit.approved_by.insert(user_id.to_hex(), edit_history_item.id);
            }
        }

        let public_audit = PublicAudit::new(&self.context, audit.clone()).await?;

        let event = PublicEvent::new(
            event_receiver,
            EventPayload::AuditUpdate(public_audit.clone()),
        );

        post_event(&self.context, event, self.context.server_auth()).await?;

        if change.report.is_some() && audit.status != AuditStatus::Resolved {
            audits.delete("_id", &id).await?;
            audits.insert(&audit).await?;
            return Ok(public_audit)
        }

        if !audit.no_customer
           && (change.report.is_some() || change.report_name.is_some() || change.action.is_some())
        {
            if let Some(chat_id) = audit.chat_id {
                delete_message(chat_id.chat_id, chat_id.message_id, auth.clone())?
            }

            let message = create_audit_message(
                CreateAuditMessage::Audit(public_audit.clone()),
                Some(audit.status.clone().into()),
                event_receiver,
                receiver_role,
                last_changer_role
            );

            let chat = send_message(message, auth)?;

            audit.chat_id = Some(AuditMessageId {
                chat_id: chat.id,
                message_id: chat.last_message.id,
            });
        }

        audits.delete("_id", &id).await?;
        audits.insert(&audit).await?;

        Ok(public_audit)
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<PublicAudit> {
        let auth = self.context.auth();

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let Some(audit) = audits.delete("_id", &id).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        if !Edit.get_access(&auth, &audit) {
            audits.insert(&audit).await?;
            return Err(anyhow::anyhow!("User is not available to delete this audit").code(403));
        }

        let public_audit = PublicAudit::new(&self.context, audit.clone()).await?;

        let (receiver_id, receiver_role, current_role) = if auth.id().unwrap() == audit.customer_id {
            (audit.auditor_id, Role::Auditor, Role::Customer)
        } else {
            (audit.customer_id, Role::Customer, Role::Auditor)
        };

        if let Some(chat_id) = audit.chat_id {
            delete_message(chat_id.chat_id, chat_id.message_id, auth.clone())?
        }

        let message = create_audit_message(
            CreateAuditMessage::Audit(public_audit.clone()),
            Some(AuditMessageStatus::Declined),
            receiver_id,
            receiver_role,
            current_role
        );

        send_message(message, auth)?;

        Ok(public_audit)
    }

    pub async fn create_issue(
        &self,
        audit_id: ObjectId,
        issue: CreateIssue,
    ) -> error::Result<PublicIssue> {
        let auth = self.context.auth();
        let Some(mut audit) = self.get_audit(audit_id).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        let issue: Issue<ObjectId> = Issue {
            id: rand::thread_rng().gen_range(10000..=99999) + audit.issues.len(),
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
            read: HashMap::new(),
        };

        audit.issues.push(issue.clone());

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        audits.delete("_id", &audit_id).await?;

        audit.issues.sort_by(|a, b| {
            severity_to_integer(&a.severity).cmp(&severity_to_integer(&b.severity))
        });

        audits.insert(&audit).await?;

        if audit.no_customer {
            return Ok(auth.public_issue(issue));
        }

        let mut new_notification: NewNotification =
            serde_json::from_str(include_str!("../../templates/audit_issue_disclosed.txt"))?;

        new_notification
            .links
            .push(format!("/audit-info/{}/customer", audit.id));

        new_notification.user_id = Some(audit.customer_id);

        let project = get_project(&self.context, audit.project_id).await?;

        let variables = vec![("audit".to_owned(), project.name)];

        send_notification(&self.context, true, true, new_notification, variables).await?;

        Ok(auth.public_issue(issue))
    }

    fn create_event(
        context: &GeneralContext,
        issue: &mut Issue<ObjectId>,
        kind: EventKind,
        message: String,
    ) {
        let event = Event {
            timestamp: Utc::now().timestamp(),
            user: context.auth().id().unwrap(),
            kind,
            message,
            id: issue.events.len(),
        };
        issue.events.push(event);
    }

    pub async fn change_issue(
        &self,
        audit_id: ObjectId,
        issue_id: usize,
        change: ChangeIssue,
    ) -> error::Result<PublicIssue> {
        let auth = self.context.auth();
        let Some(mut audit) = self.get_audit(audit_id).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        if !change.get_access(&audit, &auth) && !audit.no_customer {
            return Err(anyhow::anyhow!("User is not available to change this issue").code(403));
        }

        let Some(mut issue) = audit
            .issues
            .iter()
            .find(|issue| issue.id == issue_id)
            .cloned()
            else {
                return Err(anyhow::anyhow!("No issue found").code(404));
            };

        if let Some(name) = change.name {
            issue.name = name;

            Self::create_event(
                &self.context,
                &mut issue,
                EventKind::IssueName,
                "changed name of the issue".to_string(),
            );
        }

        if let Some(description) = change.description {
            issue.description = description;

            Self::create_event(
                &self.context,
                &mut issue,
                EventKind::IssueDescription,
                "changed description".to_string(),
            );
        }

        let role = if auth.id().unwrap() == audit.customer_id {
            Role::Customer
        } else {
            Role::Auditor
        };

        let receiver_id = if role == Role::Customer {
            audit.auditor_id
        } else {
            audit.customer_id
        };

        if let Some(action) = change.status {
            if audit.no_customer {
                issue.status = match action {
                    Action::Fixed => Status::Fixed,
                    Action::NotFixed => Status::NotFixed,
                    _ => return Err(anyhow::anyhow!("Invalid action").code(400))
                }
            } else {
                let Some(new_state) = issue.status.apply(&action) else {
                    return Err(anyhow::anyhow!("Invalid action").code(400));
                };

                let mut new_notification: NewNotification = if role == Role::Customer {
                    serde_json::from_str(include_str!(
                        "../../templates/audit_issue_status_change_auditor.txt"
                    ))?
                } else {
                    serde_json::from_str(include_str!(
                        "../../templates/audit_issue_status_change_customer.txt"
                    ))?
                };

                new_notification.user_id = Some(receiver_id);

                let project = get_project(&self.context, audit.project_id).await?;

                let variables = vec![
                    ("issue".to_owned(), issue.name.clone()),
                    ("audit".to_owned(), project.name),
                ];

                send_notification(&self.context, true, true, new_notification, variables).await?;

                issue.status = new_state.clone();

                Self::create_event(
                    &self.context,
                    &mut issue,
                    EventKind::StatusChange,
                    format!("changed status to {:?}", new_state),
                );
            };
        }

        if let Some(severity) = change.severity.clone() {
            issue.severity = severity.clone();

            Self::create_event(
                &self.context,
                &mut issue,
                EventKind::IssueSeverity,
                severity,
            );
        }

        if let Some(category) = change.category {
            issue.category = category.clone();

            Self::create_event(
                &self.context,
                &mut issue,
                EventKind::IssueCategory,
                format!("changed category to {}", category),
            );
        }

        if let Some(links) = change.links {
            let prev_links_length = issue.links.len();
            issue.links = links.clone();

            let message = if prev_links_length < links.len() {
                "added new link".to_string()
            } else {
                "deleted link".to_string()
            };

            Self::create_event(&self.context, &mut issue, EventKind::IssueLink, message);
        }

        if let Some(include) = change.include {
            issue.include = include;
        }

        if let Some(feedback) = change.feedback {
            let message = if feedback.is_empty() {
                "added feedback".to_string()
            } else {
                "changed feedback".to_string()
            };

            let kind = if feedback.is_empty() {
                EventKind::FeedbackAdded
            } else {
                EventKind::FeedbackChanged
            };

            issue.feedback = feedback;

            Self::create_event(&self.context, &mut issue, kind, message);
        }

        if !audit.no_customer {
            if let Some(events) = change.events {
                let sender_id = auth.id().unwrap();

                let project = get_project(&self.context, audit.project_id).await?;

                let role = if sender_id == audit.customer_id {
                    Role::Customer
                } else {
                    Role::Auditor
                };

                for create_event in events {
                    let event = Event {
                        timestamp: Utc::now().timestamp(),
                        user: self.context.auth().id().unwrap(),
                        kind: create_event.kind,
                        message: create_event.message,
                        id: issue.events.len(),
                    };

                    if event.kind == EventKind::Comment {
                        let mut new_notification: NewNotification = if role == Role::Customer {
                            serde_json::from_str(include_str!(
                                "../../templates/audit_issue_comment_auditor.txt"
                            ))?
                        } else {
                            serde_json::from_str(include_str!(
                                "../../templates/audit_issue_comment_customer.txt"
                            ))?
                        };

                        new_notification.user_id = Some(receiver_id);

                        let variables = vec![
                            ("audit".to_owned(), project.name.clone()),
                            ("issue".to_owned(), issue.name.clone()),
                        ];

                        send_notification(&self.context, true, true, new_notification, variables)
                            .await?;
                    }

                    issue.events.push(event);
                }

                issue
                    .read
                    .insert(sender_id.to_hex(), issue.events.len() as u64);
            }
        }

        issue.last_modified = Utc::now().timestamp();

        if let Some(idx) = audit.issues.iter().position(|issue| issue.id == issue_id) {
            audit.issues[idx] = issue.clone();
        }

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        audits.delete("_id", &audit_id).await?;

        if change.severity.is_some() {
            audit.issues.sort_by(|a, b| {
                severity_to_integer(&a.severity).cmp(&severity_to_integer(&b.severity))
            });
        }

        audits.insert(&audit).await?;

        let public_issue = auth.public_issue(issue);

        let event_reciver = if auth.id().unwrap() == audit.customer_id {
            audit.auditor_id
        } else {
            audit.customer_id
        };

        let event = PublicEvent::new(
            event_reciver,
            EventPayload::IssueUpdate {
                issue: public_issue.clone(),
                audit: audit_id.to_hex(),
            },
        );

        post_event(&self.context, event, self.context.server_auth()).await?;

        Ok(public_issue)
    }

    pub async fn disclose_all(&self, audit_id: ObjectId) -> error::Result<Vec<PublicIssue>> {
        let auth = self.context.auth();
        let audit = self.get_audit(audit_id).await?;

        // TODO: make auth

        if let Some(mut audit) = audit {
            audit.issues.iter_mut().for_each(|issue| {
                if issue.status == Status::Draft {
                    issue.status = Status::InProgress;
                    issue.last_modified = Utc::now().timestamp();
                }
            });

            let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;
            audits.delete("_id", &audit_id).await?;
            audits.insert(&audit).await?;

            let issues = audit.issues;

            let issues: Vec<PublicIssue> = issues
                .into_iter()
                .map(|i| auth.public_issue(i))
                .collect::<Vec<PublicIssue>>();

            return Ok(issues);
        }

        Ok(Vec::new())
    }

    pub async fn get_issues(&self, audit_id: ObjectId) -> error::Result<Vec<PublicIssue>> {
        let auth = self.context.auth();

        let audit = self.get_audit(audit_id).await?;

        if let Some(audit) = audit {
            if !Read.get_access(&auth, &audit) {
                return Err(anyhow::anyhow!("User is not available to read this audit").code(403));
            }

            let is_customer = auth.id().unwrap() == audit.customer_id && !audit.no_customer;

            let issues = audit.issues;

            let mut issues: Vec<PublicIssue> = issues
                .into_iter()
                .map(|i| auth.public_issue(i))
                .collect::<Vec<PublicIssue>>();

            if is_customer {
                issues.retain(|issue| issue.status != Status::Draft);
            }

            return Ok(issues);
        }

        Ok(Vec::new())
    }

    pub async fn get_issue_by_id(
        &self,
        audit_id: ObjectId,
        issue_id: usize,
    ) -> error::Result<PublicIssue> {
        let auth = self.context.auth();
        let audit = self.get_audit(audit_id).await?;

        if let Some(audit) = audit {
            let issue = audit
                .issues
                .iter()
                .find(|issue| issue.id == issue_id)
                .cloned();

            if let Some(issue) = issue {
                return Ok(auth.public_issue(issue));
            }
        }

        Err(anyhow::anyhow!("No issue found").code(404))
    }

    pub async fn delete_issue(
        &self,
        audit_id: ObjectId,
        issue_id: usize,
    ) -> error::Result<PublicIssue> {
        let auth = self.context.auth();
        let Some(mut audit) = self.get_audit(audit_id).await? else {
            return Err(anyhow::anyhow!("No audit found").code(404));
        };

        if !Edit.get_access(&auth, &audit) || !audit.no_customer {
            return Err(anyhow::anyhow!("User is not available to delete this issue").code(403));
        }

        let Some(issue) = audit
            .issues
            .iter()
            .find(|issue| issue.id == issue_id)
            .cloned()
            else {
                return Err(anyhow::anyhow!("No issue found").code(404));
            };

        audit.issues.retain(|issue| issue.id != issue_id);

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;
        audits.delete("_id", &audit_id).await?;
        audits.insert(&audit).await?;
        let public_issue = auth.public_issue(issue);

        Ok(public_issue)
    }

    pub async fn read_events(
        &self,
        audit_id: ObjectId,
        issue_id: usize,
        read: u64,
    ) -> error::Result<()> {
        let auth = self.context.auth();

        let audit = self.get_audit(audit_id).await?;

        if let Some(mut audit) = audit {
            let issue = audit
                .issues
                .iter_mut()
                .find(|issue| issue.id == issue_id);

            if let Some(issue) = issue {
                issue.read.insert(auth.id().unwrap().to_hex(), read);
            }

            let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

            audits.delete("_id", &audit_id).await?;

            audits.insert(&audit).await?;

            return Ok(());
        }

        Err(anyhow::anyhow!("No issue found").code(404))
    }

    pub async fn find_public(
        &self,
        user: ObjectId,
        role: String,
    ) -> error::Result<Vec<PublicAudit>> {
        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;

        let role = role.to_ascii_lowercase() + "_id";

        if role != "customer_id" && role != "auditor_id" {
            return Err(anyhow::anyhow!("Invalid role").code(400));
        }

        let audits = audits.find_many(&role, &Bson::ObjectId(user)).await?;

        let mut result = vec![];

        for audit in audits {
            result.push(PublicAudit::new(&self.context, audit).await?);
        }

        Ok(result)
    }

    pub async fn get_audit_edit_history(
        &self,
        id: ObjectId,
    ) -> error::Result<EditHistoryResponse> {
        let Some(audit) = self.get_audit(id).await? else {
            return Err(anyhow::anyhow!("Audit not found").code(404));
        };

        let mut result = vec![];

        for history in audit.edit_history {
            let role = if history.author == audit.auditor_id.to_hex() {
                Role::Auditor
            } else {
                Role::Customer
            };
            result.push(PublicAuditEditHistory::new(&self.context, history, role).await?);
        }

        result.reverse();
        Ok(EditHistoryResponse {
            edit_history: result,
            approved_by: audit.approved_by,
        })
    }

    pub async fn change_audit_edit_history(
        &self,
        audit_id: ObjectId,
        history_id: usize,
        change: ChangeAuditHistory,
    ) -> error::Result<PublicAuditEditHistory> {
        let auth = self.context.auth();
        let user_id = auth.id().unwrap();

        let Some(mut audit) = self.get_audit(audit_id).await? else {
            return Err(anyhow::anyhow!("Audit not found").code(404));
        };

        let Some(mut history) = audit
            .edit_history
            .iter()
            .find(|h| h.id == history_id)
            .cloned()
            else {
                return Err(anyhow::anyhow!("History not found").code(404));
            };

        if let Some(comment) = change.comment {
            if user_id == history.author.parse()? {
                history.comment = Some(comment);
            } else {
                return Err(anyhow::anyhow!("Only the author of the edit can change a comment").code(403))
            }
        }

        if let Some(is_approved) = change.is_approved {
            if is_approved {
                audit.approved_by.insert(user_id.to_hex(), history_id);

                let is_last = audit
                    .edit_history
                    .last()
                    .map_or(false, |last| last.id == history_id);

                let is_approve_equal = {
                    let first = audit.approved_by.values().next().unwrap();
                    audit.approved_by.values().all(|v| v == first)
                };

                if !is_last && is_approve_equal {
                    let new_history_item = AuditEditHistory {
                        id: audit.edit_history.len(),
                        date: Utc::now().timestamp_micros(),
                        author: user_id.to_hex(),
                        comment: None,
                        audit: history.audit.clone(),
                    };
                    audit.edit_history.push(new_history_item);

                    let audit_change: AuditChange = serde_json::from_str(&history.audit).unwrap();
                    let updated_audit = self.change(audit_id, audit_change).await?;
                    audit.project_name = updated_audit.project_name;
                    audit.description = updated_audit.description;
                    audit.scope = updated_audit.scope;
                    audit.tags = updated_audit.tags;
                    audit.price = updated_audit.price;
                    audit.total_cost = updated_audit.total_cost;
                    audit.time = updated_audit.time;
                }
            }
        }

        if let Some(idx) = audit.edit_history.iter().position(|h| h.id == history_id) {
            audit.edit_history[idx] = history.clone();
        }

        let audits = self.context.try_get_repository::<Audit<ObjectId>>()?;
        audits.delete("_id", &audit_id).await?;
        audits.insert(&audit).await?;

        let role = if history.author == audit.auditor_id.to_hex() {
            Role::Auditor
        } else {
            Role::Customer
        };

        Ok(PublicAuditEditHistory::new(&self.context, history, role).await?)
    }
}
