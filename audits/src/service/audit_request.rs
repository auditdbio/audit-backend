use chrono::Utc;
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};
use serde_json::json;

use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{
        auditor::request_auditor,
        badge::BadgePayload,
        codes::post_code,
        chat::{
            create_audit_message, send_message,
            delete_message, AuditMessageStatus,
            CreateAuditMessage, AuditMessageId
        },
        events::{EventPayload, PublicEvent},
        mail::send_mail,
        requests::CreateRequest,
        seartch::PaginationParams,
        send_notification, NewNotification,
    },
    context::GeneralContext,
    entities::{
        audit_request::{AuditRequest, TimeRange},
        audit::{AuditEditHistory, PublicAuditEditHistory},
        auditor::ExtendedAuditor,
        letter::CreateLetter,
        project::get_project,
        role::Role,
    },
    error::{self, AddCode},
    services::{API_PREFIX, CUSTOMERS_SERVICE, EVENTS_SERVICE, FRONTEND, PROTOCOL},
};

pub use common::api::requests::PublicRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestChange {
    description: Option<String>,
    time: Option<TimeRange>,
    scope: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    price: Option<i64>,
    total_cost: Option<i64>,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyAuditRequestResult {
    pub result: Vec<PublicRequest>,
    #[serde(rename = "totalDocuments")]
    pub total_documents: u64,
}

pub struct RequestService {
    context: GeneralContext,
}

impl RequestService {
    #[must_use]
    pub const fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn create(&self, request: CreateRequest) -> error::Result<PublicRequest> {
        let auth = self.context.auth();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(user_id) = auth.id() else {
            return Err(
                anyhow::anyhow!("Audit can be created only by authenticated user").code(400),
            );
        };

        let customer_id = request.customer_id.parse()?;
        let auditor_id = request.auditor_id.parse()?;

        if customer_id == auditor_id {
            return Err(anyhow::anyhow!("You can't create audit with yourself").code(400));
        }

        let last_changer = if user_id == customer_id {
            Role::Customer
        } else if user_id == auditor_id {
            Role::Auditor
        } else {
            return Err(
                anyhow::anyhow!("Audit can be created only by customer or auditor").code(400),
            );
        };

        let price_per_line = if request.total_cost.is_none() {
            request.price
        } else {
            None
        };

        let project = get_project(&self.context, request.project_id.parse()?).await?;

        let mut request = AuditRequest {
            id: ObjectId::new(),
            customer_id,
            auditor_id,
            project_id: request.project_id.parse()?,
            description: project.description,
            tags: Some(project.tags),
            scope: Some(project.scope),
            time: request.time,
            price: price_per_line,
            total_cost: request.total_cost,
            last_modified: Utc::now().timestamp_micros(),
            last_changer,
            chat_id: None,
            edit_history: Vec::new(),
        };

        let edit_history_item = AuditEditHistory {
            id: request.edit_history.len(),
            date: request.last_modified.clone(),
            author: project.customer_id.clone(),
            comment: None,
            audit: serde_json::to_string(&json!({
                    "project_name": project.name,
                    "description": request.description,
                    "scope": request.scope,
                    "tags": request.tags,
                    "price": request.price,
                    "total_cost": request.total_cost,
                    "time": request.time,
                    "conclusion": "".to_string(),
                })).unwrap(),
        };

        request.edit_history.push(edit_history_item);

        let old_version_of_this_request = requests
            .find_many("project_id", &Bson::ObjectId(request.project_id))
            .await?
            .into_iter()
            .filter(|r| r.customer_id == request.customer_id && r.auditor_id == request.auditor_id)
            .collect::<Vec<_>>()
            .pop();

        if let Some(old_version_of_this_request) = old_version_of_this_request {
            requests
                .delete("id", &old_version_of_this_request.id)
                .await?;
            request.id = old_version_of_this_request.id;
            request.chat_id = old_version_of_this_request.chat_id;
        } else if last_changer == Role::Customer {
            let mut new_notification: NewNotification =
                serde_json::from_str(include_str!("../../templates/new_audit_request.txt"))?;

            new_notification
                .links
                .push(format!("/audit-request/{}", request.id));

            new_notification.user_id = Some(request.auditor_id);

            let variables: Vec<(String, String)> =
                vec![("project".to_owned(), project.name.clone())];

            if let Err(err) =
                send_notification(&self.context, true, true, new_notification, variables).await
            {
                log::warn!("Failed to send notification: {}", err); // TODO: this always fails for badges, do something with it
            }
        } else {
            let mut new_notification: NewNotification =
                serde_json::from_str(include_str!("../../templates/new_audit_offer.txt"))?;

            new_notification
                .links
                .push(format!("/audit-request/{}/customer", request.id));

            new_notification.user_id = Some(request.customer_id);

            let variables: Vec<(String, String)> =
                vec![("project".to_owned(), project.name.clone())];

            send_notification(&self.context, true, true, new_notification, variables).await?;
        }

        if last_changer == Role::Customer {
            self.context
                .make_request::<()>()
                .auth(auth)
                .post(format!(
                    "{}://{}/project/auditor/{}/{}",
                    PROTOCOL.as_str(),
                    CUSTOMERS_SERVICE.as_str(),
                    request.project_id,
                    request.auditor_id
                ))
                .send()
                .await?;
        }

        if let ExtendedAuditor::Badge(badge) = request_auditor(
            &self.context,
            request.auditor_id,
            self.context.server_auth(),
        )
        .await?
        {
            let payload = BadgePayload {
                badge_id: badge.user_id.parse()?,
                email: badge.contacts.email.clone().unwrap(),
            };

            let code = post_code(&self.context, serde_json::to_string(&payload)?).await?;

            // delete link
            let delete_link = format!(
                "{}://{}/delete/{}/{}",
                PROTOCOL.as_str(),
                FRONTEND.as_str(),
                badge.user_id,
                code
            );
            // merge link
            let merge_link = format!(
                "{}://{}/invite-user/{}/{}",
                PROTOCOL.as_str(),
                FRONTEND.as_str(),
                badge.user_id,
                code
            );

            let message = include_str!("../../templates/badge_link.txt")
                .replace("{merge_link}", &merge_link)
                .replace("{delete_link}", &delete_link);

            // send email
            let letter = CreateLetter {
                recipient_id: None,
                recipient_name: None,
                email: badge.contacts.email.unwrap(),
                message: message.clone(),
                subject: "AuditDB Audit Request".to_string(),
            };
            send_mail(&self.context, letter).await?;
        }

        let (event_receiver, receiver_role) = if last_changer == Role::Customer {
            (auditor_id, Role::Auditor)
        } else {
            (customer_id, Role::Customer)
        };

        let public_request = PublicRequest::new(&self.context, request.clone()).await?;

        if let Some(chat_id) = request.chat_id {
            delete_message(chat_id.chat_id, chat_id.message_id, auth.clone())?
        }

        let message = create_audit_message(
            CreateAuditMessage::Request(public_request.clone()),
            Some(AuditMessageStatus::Request),
            event_receiver,
            receiver_role,
            last_changer
        );

        let chat = send_message(message, auth)?;

        request.chat_id = Some(AuditMessageId {
            chat_id: chat.id,
            message_id: chat.last_message.id,
        });

        requests.insert(&request).await?;

        let event = PublicEvent::new(
            event_receiver,
            EventPayload::NewRequest(public_request.clone()),
        );

        self.context
            .make_request()
            .post(format!(
                "{}://{}/{}/event",
                PROTOCOL.as_str(),
                EVENTS_SERVICE.as_str(),
                API_PREFIX.as_str(),
            ))
            .auth(self.context.server_auth())
            .json(&event)
            .send()
            .await?;

        Ok(public_request)
    }

    async fn get_request(&self, id: ObjectId) -> error::Result<Option<AuditRequest<ObjectId>>> {
        let auth = self.context.auth();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(request) = requests.find("id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(&auth, &request) {
            return Err(anyhow::anyhow!("User is not available to read this request").code(400));
        }

        Ok(Some(request))
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicRequest>> {
        let request = self.get_request(id).await?;

        if let Some(request) = request {
            let public_request = PublicRequest::new(&self.context, request).await?;
            return Ok(Some(public_request));
        }

        Ok(None)
    }

    pub async fn my_request(
        &self,
        role: Role,
        pagination: PaginationParams,
    ) -> error::Result<Vec<PublicRequest>> {
        let page = pagination.page.unwrap_or(0);
        let per_page = pagination.per_page.unwrap_or(0);
        let limit = pagination.per_page.unwrap_or(1000);
        let skip = (page - 1) * per_page;

        let auth = self.context.auth();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(user_id) = auth.id() else {
            return Err(
                anyhow::anyhow!("Audit can be created only by authenticated user").code(400),
            );
        };

        let id = match role {
            Role::Auditor => "auditor_id",
            Role::Customer => "customer_id",
        };

        let (result, _total_documents) = requests
            .find_many_limit(id, &Bson::ObjectId(user_id), skip, limit)
            .await?;

        let mut public_requests = Vec::new();

        for req in result {
            let public_request = PublicRequest::new(&self.context, req).await?;

            public_requests.push(public_request);
        }

        // Ok(MyAuditRequestResult {
        //     result: public_requests,
        //     total_documents,
        // })
        Ok(public_requests)
    }

    pub async fn change(
        &self,
        id: ObjectId,
        change: RequestChange,
    ) -> error::Result<PublicRequest> {
        let auth = self.context.auth();
        let user_id = auth.id().unwrap();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(mut request) = requests.find("id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No customer found").code(404));
        };

        if !Edit.get_access(&auth, &request) {
            return Err(anyhow::anyhow!("User is not available to change this audit request").code(400));
        }

        let mut is_history_changed = false;

        if let Some(description) = change.description {
            if request.description != description {
                request.description = description;
                is_history_changed = true;
            }
        }

        if change.scope.is_some() {
            if request.scope != change.scope {
                request.scope = change.scope;
                is_history_changed = true;
            }
        }

        if change.tags.is_some() {
            if request.tags != change.tags {
                request.tags = change.tags;
                is_history_changed = true;
            }
        }

        if let Some(time) = change.time {
            request.time = time;
            is_history_changed = true;
        }

        if change.total_cost.is_some() {
            if request.total_cost != change.total_cost {
                request.total_cost = change.total_cost;
                request.price = None;
                is_history_changed = true;
            }
        }

        if change.price.is_some() && change.total_cost.is_none() {
            if request.price != change.price {
                request.price = change.price;
                request.total_cost = None;
                is_history_changed = true;
            }
        }

        let last_changer_role = if user_id == request.customer_id {
            Role::Customer
        } else if user_id == request.auditor_id {
            Role::Auditor
        } else {
            return Err(anyhow::anyhow!("User is not available to change this request").code(400));
        };

        let (receiver_id, receiver_role) = if last_changer_role == Role::Customer {
            (request.auditor_id, Role::Auditor)
        } else {
            (request.customer_id, Role::Customer)
        };

        request.last_changer = last_changer_role;

        request.last_modified = Utc::now().timestamp_micros();

        if is_history_changed {
            let project = get_project(&self.context, request.project_id).await?;

            let edit_history_item = AuditEditHistory {
                id: request.edit_history.len(),
                date: request.last_modified.clone(),
                author: user_id.to_hex(),
                comment: change.comment,
                audit: serde_json::to_string(&json!({
                    "project_name": project.name,
                    "description": request.description,
                    "scope": request.scope,
                    "tags": request.tags,
                    "price": request.price,
                    "total_cost": request.total_cost,
                    "time": request.time,
                    "conclusion": "".to_string(),
                })).unwrap(),
            };

            request.edit_history.push(edit_history_item);
        }

        let public_request = PublicRequest::new(&self.context, request.clone()).await?;

        if let Some(chat_id) = request.chat_id {
            delete_message(chat_id.chat_id, chat_id.message_id, auth.clone())?
        }

        let message = create_audit_message(
            CreateAuditMessage::Request(public_request.clone()),
            Some(AuditMessageStatus::Request),
            receiver_id,
            receiver_role,
            last_changer_role,
        );

        let chat = send_message(message, auth)?;

        request.chat_id = Some(AuditMessageId {
            chat_id: chat.id,
            message_id: chat.last_message.id,
        });

        requests.delete("id", &id).await?;
        requests.insert(&request).await?;

        Ok(public_request)
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<PublicRequest> {
        let auth = self.context.auth();

        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let Some(request) = requests.delete("id", &id).await? else {
            return Err(anyhow::anyhow!("No customer found").code(404));
        };

        if !Edit.get_access(&auth, &request) {
            requests.insert(&request).await?;
            return Err(anyhow::anyhow!("User is not available to delete this request").code(400));
        }

        let current_role = if auth.id() == Some(request.customer_id) {
            Role::Customer
        } else if auth.id() == Some(request.auditor_id) {
            Role::Auditor
        } else {
            return Err(anyhow::anyhow!("User is not available to delete this request").code(400));
        };

        let public_request = PublicRequest::new(&self.context, request.clone()).await?;

        if current_role == Role::Customer {
            self.context
                .make_request::<()>()
                .auth(auth)
                .post(format!(
                    "{}://{}/project/auditor/{}/{}",
                    PROTOCOL.as_str(),
                    CUSTOMERS_SERVICE.as_str(),
                    request.project_id,
                    request.auditor_id
                ))
                .send()
                .await?;
        }

        let (event_receiver, receiver_role) = if current_role == Role::Customer {
            (public_request.auditor_id.parse::<ObjectId>()?, Role::Auditor)
        } else {
            (public_request.customer_id.parse::<ObjectId>()?, Role::Customer)
        };

        let event = PublicEvent::new(
            event_receiver,
            EventPayload::RequestDecline(public_request.id.clone()),
        );

        self.context
            .make_request()
            .post(format!(
                "{}://{}/{}/event",
                PROTOCOL.as_str(),
                EVENTS_SERVICE.as_str(),
                API_PREFIX.as_str(),
            ))
            .auth(self.context.server_auth())
            .json(&event)
            .send()
            .await?;

        if let Some(chat_id) = request.chat_id {
            delete_message(chat_id.chat_id, chat_id.message_id, auth.clone())?
        }

        let message = create_audit_message(
            CreateAuditMessage::Request(public_request.clone()),
            Some(AuditMessageStatus::Declined),
            event_receiver,
            receiver_role,
            current_role,
        );

        send_message(message, auth)?;

        Ok(public_request)
    }

    pub async fn find_all(
        &self,
        role: Role,
        user_id: ObjectId,
    ) -> error::Result<Vec<AuditRequest<String>>> {
        let requests = self
            .context
            .try_get_repository::<AuditRequest<ObjectId>>()?;

        let id = match role {
            Role::Auditor => "auditor_id",
            Role::Customer => "customer_id",
        };

        let result = requests
            .find_many(id, &Bson::ObjectId(user_id))
            .await?
            .into_iter()
            .map(|r| r.stringify())
            .collect();

        Ok(result)
    }

    pub async fn get_request_edit_history(
        &self,
        id: ObjectId,
    ) -> error::Result<Vec<PublicAuditEditHistory>> {
        let Some(request) = self.get_request(id).await? else {
            return Err(anyhow::anyhow!("Audit request not found").code(404));
        };

        let mut result = vec![];

        for history in request.edit_history {
            let role = if history.author == request.auditor_id.to_hex() {
                Role::Auditor
            } else {
                Role::Customer
            };
            result.push(PublicAuditEditHistory::new(&self.context, history, role).await?);
        }

        result.reverse();
        Ok(result)
    }
}
