use std::{collections::HashMap};

use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{audit::Audit, audit_request::{TimeRange, AuditRequest}, project::PublicProject, role::Role},
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
    pub report: Option<String>,
    pub time: Option<TimeRange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicAudit {
    pub id: String,
    pub auditor_id: String,
    pub customer_id: String,
    pub project_id: String,
    pub project_name: String,
    pub avatar: String,
    pub description: String,
    pub status: String,
    pub scope: Vec<String>,
    pub price: i64,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub tags: Vec<String>,
    pub last_modified: i64,
    pub report: Option<String>,
    pub time: TimeRange,
}

impl From<Audit<ObjectId>> for PublicAudit {
    fn from(audit: Audit<ObjectId>) -> Self {
        Self {
            id: audit.id.to_hex(),
            customer_id: audit.customer_id.to_hex(),
            auditor_id: audit.auditor_id.to_hex(),
            project_id: audit.project_id.to_hex(),
            project_name: audit.project_name,
            avatar: audit.avatar,
            description: audit.description,
            status: audit.status,
            scope: audit.scope,
            price: audit.price,
            auditor_contacts: audit.auditor_contacts,
            customer_contacts: audit.customer_contacts,
            tags: audit.tags,
            last_modified: audit.last_modified,
            report: audit.report,
            time: audit.time,
        }
    }
}

pub struct AuditService {
    context: Context,
}

impl AuditService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, request: PublicRequest) -> anyhow::Result<PublicAudit> {
        let Some(audits) = self.context.get_repository::<Audit<ObjectId>>() else {
            bail!("No audit repository found")
        };

        let project = self
            .context
            .make_request::<PublicProject>()
            .get(format!(
                "{}://{}/api/project/{}",
                PROTOCOL.as_str(),
                CUSTOMERS_SERVICE.as_str(),
                request.project_id
            ))
            .auth(self.context.server_auth())
            .send()
            .await?
            .json::<PublicProject>()
            .await?;

        let audit = Audit {
            id: ObjectId::new(),
            customer_id: request.customer_id.parse()?,
            auditor_id: request.auditor_id.parse()?,
            project_id: request.project_id.parse()?,
            project_name: request.project_name,
            avatar: request.project_avatar,
            description: request.description,
            status: "pending".to_string(),
            scope: request.project_scope,
            price: request.price,
            auditor_contacts: request.auditor_contacts,
            customer_contacts: request.customer_contacts,
            tags: project.tags,
            last_modified: Utc::now().timestamp_micros(),
            report: None,
            time: request.time,
        };

        audits.insert(&audit).await?;

        let Some(requests) = self.context.get_repository::<AuditRequest<ObjectId>>() else {
            bail!("No audit request repository found")
        };

        requests
            .delete("id", &request.id.parse()?)
            .await?;

        Ok(audit.into())
    }

    pub async fn find(&self, id: ObjectId) -> anyhow::Result<Option<PublicAudit>> {
        let auth = self.context.auth();

        let Some(audits) = self.context.get_repository::<Audit<ObjectId>>() else {
            bail!("No audit repository found")
        };

        let Some(audit) = audits.find("id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read::get_access(auth, &audit) {
            bail!("User is not available to change this audit")
        }

        Ok(Some(audit.into()))
    }

    pub async fn my_audit(&self, role: Role) -> anyhow::Result<Vec<Audit<String>>> {
        let auth = self.context.auth();

        let Some(audits) = self.context.get_repository::<Audit<ObjectId>>() else {
            bail!("No audit repository found")
        };

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

        Ok(audits.into_iter().map(Audit::stringify).collect())
    }

    pub async fn change(&self, id: ObjectId, change: AuditChange) -> anyhow::Result<PublicAudit> {
        let auth = self.context.auth();

        let Some(audits) = self.context.get_repository::<Audit<ObjectId>>() else {
            bail!("No audit repository found")
        };

        let Some(mut audit) = audits.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No audit found")
        };

        if !Edit::get_access(auth, &audit) {
            bail!("User is not available to change this audit")
        }

        if let Some(avatar) = change.avatar {
            audit.avatar = avatar;
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

        if let Some(time) = change.time {
            audit.time = time;
        }

        audit.last_modified = Utc::now().timestamp_micros();

        audits.delete("id", &id).await?;
        audits.insert(&audit).await?;

        Ok(audit.into())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicAudit> {
        let auth = self.context.auth();

        let Some(audits) = self.context.get_repository::<Audit<ObjectId>>() else {
            bail!("No audit repository found")
        };

        let Some(audit) = audits.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No audit found")
        };

        if !Edit::get_access(auth, &audit) {
            audits.insert(&audit).await?;
            bail!("User is not available to delete this audit")
        }

        Ok(audit.into())
    }
}
