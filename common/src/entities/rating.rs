use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    api::{audits::PublicAudit, user::get_by_id},
    context::GeneralContext,
    entities::{
        audit::PublicAuditStatus,
        audit_request::TimeRange,
        role::Role,
    },
    error,
    repository::Entity,
    services::{AUDITS_SERVICE, API_PREFIX, PROTOCOL},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rating<Id> {
    pub id: Id,
    pub auditor_id: Id,
    pub last_update: i64,
    pub summary: u8,
    pub user_feedbacks: Vec<UserFeedback<Id>>,
    pub total_completed_audits: Vec<CompletedAuditInfo<Id>>,
}

impl Rating<ObjectId> {
    pub fn stringify(self) -> Rating<String> {
        Rating {
            id: self.id.to_hex(),
            auditor_id: self.auditor_id.to_hex(),
            last_update: self.last_update,
            summary: self.summary,
            user_feedbacks: UserFeedback::stringify_map(self.user_feedbacks),
            total_completed_audits: CompletedAuditInfo::stringify_map(self.total_completed_audits),
        }
    }

    pub async fn calculate(
        &mut self,
        context: &GeneralContext,
        auditor_id: ObjectId
    ) -> error::Result<Rating<ObjectId>> {
        let audits = context
            .make_request::<Vec<PublicAudit>>()
            .get(format!(
                "{}://{}/{}/public_audits/{}/auditor",
                PROTOCOL.as_str(),
                AUDITS_SERVICE.as_str(),
                API_PREFIX.as_str(),
                auditor_id,
            ))
            .auth(context.server_auth())
            .send()
            .await?
            .json::<Vec<PublicAudit>>()
            .await?;

        self.total_completed_audits = audits
            .into_iter()
            .filter(|a| a.status == PublicAuditStatus::Resolved && a.resolved_at.is_some())
            .map(|a| CompletedAuditInfo {
                audit_id: a.id.parse().unwrap(),
                // project_name: a.project_name,
                completed_at: a.resolved_at.unwrap(),
                time: a.time,
            })
            .collect::<Vec<CompletedAuditInfo<ObjectId>>>();

        let user = get_by_id(
            &context,
            context.server_auth(),
            auditor_id,
        ).await?;

        let _identity = user.linked_accounts.as_ref().map_or(0, Vec::len);


        // TODO CALC FUNC

        self.last_update = Utc::now().timestamp_micros();

        Ok(self.clone())
    }
}

impl Rating<String> {
    pub fn parse(self) -> Rating<ObjectId> {
        Rating {
            id: self.id.parse().unwrap(),
            auditor_id: self.auditor_id.parse().unwrap(),
            last_update: self.last_update,
            summary: self.summary,
            user_feedbacks: UserFeedback::parse_map(self.user_feedbacks),
            total_completed_audits: CompletedAuditInfo::parse_map(self.total_completed_audits),
        }
    }
}

impl Entity for Rating<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletedAuditInfo<Id> {
    pub audit_id: Id,
    // pub project_name: String,
    pub completed_at: i64,
    pub time: TimeRange,
}

impl CompletedAuditInfo<String> {
    pub fn parse(self) -> CompletedAuditInfo<ObjectId> {
        CompletedAuditInfo {
            audit_id: self.audit_id.parse().unwrap(),
            // project_name: self.project_name,
            completed_at: self.completed_at,
            time: self.time,
        }
    }

    pub fn parse_map(map: Vec<Self>) -> Vec<CompletedAuditInfo<ObjectId>> {
        map.into_iter().map(|v| v.parse()).collect()
    }
}

impl CompletedAuditInfo<ObjectId> {
    pub fn stringify(self) -> CompletedAuditInfo<String> {
        CompletedAuditInfo {
            audit_id: self.audit_id.to_hex(),
            // project_name: self.project_name,
            completed_at: self.completed_at,
            time: self.time,
        }
    }

    pub fn stringify_map(map: Vec<Self>) -> Vec<CompletedAuditInfo<String>> {
        map.into_iter().map(|v| v.stringify()).collect()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserFeedbackRating {
    pub quality_of_work: u8,
    pub time_management: Option<u8>,
    pub collaboration: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedbackFrom<Id> {
    pub user_id: Id,
    pub role: Role,
}

impl FeedbackFrom<String> {
    pub fn parse(self) -> FeedbackFrom<ObjectId> {
        FeedbackFrom {
            user_id: self.user_id.parse().unwrap(),
            role: self.role,
        }
    }
}

impl FeedbackFrom<ObjectId> {
    pub fn stringify(self) -> FeedbackFrom<String> {
        FeedbackFrom {
            user_id: self.user_id.to_hex(),
            role: self.role,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserFeedback<Id> {
    pub id: Id,
    pub audit_id: Id,
    pub from: FeedbackFrom<Id>,
    pub created_at: i64,
    pub rating: UserFeedbackRating,
    pub comment: Option<String>,
}

impl UserFeedback<String> {
    pub fn parse(self) -> UserFeedback<ObjectId> {
        UserFeedback {
            id: self.id.parse().unwrap(),
            audit_id: self.audit_id.parse().unwrap(),
            from: self.from.parse(),
            created_at: self.created_at,
            rating: self.rating,
            comment: self.comment,
        }
    }

    pub fn parse_map(map: Vec<Self>) -> Vec<UserFeedback<ObjectId>> {
        map.into_iter().map(|v| v.parse()).collect()
    }
}

impl UserFeedback<ObjectId> {
    pub fn stringify(self) -> UserFeedback<String> {
        UserFeedback {
            id: self.id.to_hex(),
            audit_id: self.audit_id.to_hex(),
            from: self.from.stringify(),
            created_at: self.created_at,
            rating: self.rating,
            comment: self.comment,
        }
    }

    pub fn stringify_map(map: Vec<Self>) -> Vec<UserFeedback<String>> {
        map.into_iter().map(|v| v.stringify()).collect()
    }
}
