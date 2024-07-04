use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    api::{audits::PublicAudit, user::get_by_id},
    context::GeneralContext,
    entities::{
        audit::PublicAuditStatus,
        audit_request::TimeRange,
        role::Role,
    },
    error,
    repository::{Entity, HasLastModified},
    services::{AUDITS_SERVICE, AUDITORS_SERVICE, API_PREFIX, CUSTOMERS_SERVICE, PROTOCOL},
    default_timestamp, impl_has_last_modified,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rating<Id> {
    pub id: Id,
    pub user_id: Id,
    pub auditor: RoleRating<Id>,
    pub customer: RoleRating<Id>,
    #[serde(default = "default_timestamp")]
    pub last_modified: i64,
}

impl_has_last_modified!(Rating<ObjectId>);

impl Rating<ObjectId> {
    pub fn stringify(self) -> Rating<String> {
        Rating {
            id: self.id.to_hex(),
            user_id: self.user_id.to_hex(),
            auditor: self.auditor.stringify(),
            customer: self.customer.stringify(),
            last_modified: self.last_modified,
        }
    }

    pub async fn calculate(
        &self,
        context: &GeneralContext,
        role: Role,
    ) -> error::Result<Rating<ObjectId>> {
        let mut rating = self.clone();

        let audits = context
            .make_request::<Vec<PublicAudit>>()
            .get(format!(
                "{}://{}/{}/public_audits/{}/{}",
                PROTOCOL.as_str(),
                AUDITS_SERVICE.as_str(),
                API_PREFIX.as_str(),
                rating.user_id,
                role.stringify(),
            ))
            .auth(context.server_auth())
            .send()
            .await?
            .json::<Vec<PublicAudit>>()
            .await?;

        let user = get_by_id(
            &context,
            context.server_auth(),
            rating.user_id,
        ).await?;


        // Rating calculation:
        const IDENTITY_POINT: f32 = 5.0;
        const LAST_COMPLETED_AUDITS_MULTIPLIER: f32 = 1.5;
        const FEEDBACK_MULTIPLIER: f32 = 12.0;

        const COMPLETED_IN_TIME_MAX_POINTS: u8 = 15;
        const LAST_COMPLETED_MAX_POINTS: u8 = 15;
        const IDENTITY_MAX_POINTS: u8 = 10;
        const FEEDBACK_MAX_POINTS: u8 = 60;

        let identity_points = user
            .linked_accounts
            .as_ref()
            .map_or(0.0, |a| a.len().min(2) as f32) * IDENTITY_POINT;

        let total_completed_audits = audits
            .into_iter()
            .filter(|a| a.status == PublicAuditStatus::Resolved && !a.no_customer)
            .map(|a| CompletedAuditInfo {
                audit_id: a.id.parse().unwrap(),
                project_name: a.project_name,
                completed_at: a.resolved_at.unwrap_or(a.time.to.clone() * 1000),
                time: a.time,
            })
            .collect::<Vec<CompletedAuditInfo<ObjectId>>>();

        let completed_in_time_audits = total_completed_audits
            .clone()
            .into_iter()
            .filter(|a| a.completed_at < a.time.to * 1000)
            .collect::<Vec<CompletedAuditInfo<ObjectId>>>();

        let completed_in_time_points = if total_completed_audits.is_empty() {
            0.0
        } else {
            (completed_in_time_audits.len() as f32 / total_completed_audits.len() as f32) * COMPLETED_IN_TIME_MAX_POINTS as f32
        };

        let last_completed_audits = total_completed_audits
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect::<Vec<CompletedAuditInfo<ObjectId>>>();

        let time_now = Utc::now().timestamp_micros();
        const MICROS_IN_A_YEAR: f32 = 365.25 * 24.0 * 60.0 * 60.0 * 1_000_000.0;

        let mut last_completed_audits_points: f32 = 0.0;
        for audit in last_completed_audits {
            let duration: f32 = (time_now - audit.completed_at) as f32;
            let years_passed = (duration / MICROS_IN_A_YEAR).ceil();
            let point = if years_passed < 2.0 {
                1.0
            } else {
                (1.0 - (years_passed - 2.0) * 0.1).max(0.1)
            };
            last_completed_audits_points += point;
        }
        last_completed_audits_points *= LAST_COMPLETED_AUDITS_MULTIPLIER;

        let user_feedbacks = match role {
            Role::Auditor => rating.clone().auditor.user_feedbacks,
            Role::Customer => rating.clone().customer.user_feedbacks,
        };

        let mut feedback_points: f32 = 0.0;

        if !user_feedbacks.is_empty() {
            for feedback in user_feedbacks.clone() {
                let mut sum = 0;
                let mut quantity = 0;
                if let Some(quality_of_work) = feedback.rating.quality_of_work {
                    sum += quality_of_work.as_i32();
                    quantity += 1;
                }
                if let Some(time_management) = feedback.rating.time_management {
                    sum += time_management.as_i32();
                    quantity += 1;
                }
                if let Some(collaboration) = feedback.rating.collaboration {
                    sum += collaboration.as_i32();
                    quantity += 1;
                }

                let duration: f32 = (time_now - feedback.created_at) as f32;
                let years_passed = (duration / MICROS_IN_A_YEAR).ceil();
                let mut point = sum as f32 / quantity as f32;

                if years_passed > 2.0 {
                    point = (point - (years_passed - 2.0) * 0.1).max(0.1)
                }

                feedback_points += point;
            }

            feedback_points = feedback_points / user_feedbacks.len() as f32 * FEEDBACK_MULTIPLIER;
        }

        let mut summary = identity_points + completed_in_time_points + last_completed_audits_points + feedback_points;
        summary = (summary * 10.0).trunc() / 10.0;

        let rating_details = serde_json::to_string(&json!({
            "Identity points": format!("{} out of {}", identity_points, IDENTITY_MAX_POINTS),
            "Completed in time points": format!(
                "{} out of {}",
                (completed_in_time_points * 10.0).trunc() / 10.0,
                COMPLETED_IN_TIME_MAX_POINTS,
            ),
            "Last completed audits points": format!(
                "{} out of {}",
                (last_completed_audits_points * 10.0).trunc() / 10.0,
                LAST_COMPLETED_MAX_POINTS,
            ),
            "Feedback points": format!(
                "{} out of {}",
                (feedback_points * 10.0).trunc() / 10.0,
                FEEDBACK_MAX_POINTS,
            ),
        })).unwrap();

        let patch_url: String;

        match role {
            Role::Auditor => {
                rating.auditor.total_completed_audits = total_completed_audits;
                rating.auditor.last_update = Utc::now().timestamp_micros();
                rating.auditor.summary = summary.clone();
                rating.auditor.rating_details = Some(rating_details);

                patch_url = format!(
                    "{}://{}/{}/auditor/{}",
                    PROTOCOL.as_str(),
                    CUSTOMERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    rating.user_id,
                )
            },
            Role::Customer => {
                rating.customer.total_completed_audits = total_completed_audits;
                rating.customer.last_update = Utc::now().timestamp_micros();
                rating.customer.summary = summary.clone();
                rating.customer.rating_details = Some(rating_details);

                patch_url = format!(
                    "{}://{}/{}/customer/{}",
                    PROTOCOL.as_str(),
                    AUDITORS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    rating.user_id,
                )
            }
        }

        context
            .make_request()
            .patch(patch_url)
            .auth(context.server_auth())
            .json(&json!({
                "rating": summary
            }))
            .send()
            .await?;

        Ok(rating)
    }
}

impl Rating<String> {
    pub fn parse(self) -> Rating<ObjectId> {
        Rating {
            id: self.id.parse().unwrap(),
            user_id: self.user_id.parse().unwrap(),
            auditor: self.auditor.parse(),
            customer: self.customer.parse(),
            last_modified: self.last_modified,
        }
    }
}

impl Entity for Rating<ObjectId> {
    fn id(&self) -> ObjectId {
        self.id
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleRating<Id> {
    pub last_update: i64,
    pub summary: f32,
    pub rating_details: Option<String>,
    pub user_feedbacks: Vec<UserFeedback<Id>>,
    pub total_completed_audits: Vec<CompletedAuditInfo<Id>>,
}

impl RoleRating<String> {
    pub fn parse(self) -> RoleRating<ObjectId> {
        RoleRating {
            last_update: self.last_update,
            summary: self.summary,
            rating_details: self.rating_details,
            user_feedbacks: UserFeedback::parse_map(self.user_feedbacks),
            total_completed_audits: CompletedAuditInfo::parse_map(self.total_completed_audits),
        }
    }
}

impl RoleRating<ObjectId> {
    pub fn stringify(self) -> RoleRating<String> {
        RoleRating {
            last_update: self.last_update,
            summary: self.summary,
            rating_details: self.rating_details,
            user_feedbacks: UserFeedback::stringify_map(self.user_feedbacks),
            total_completed_audits: CompletedAuditInfo::stringify_map(self.total_completed_audits),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletedAuditInfo<Id> {
    pub audit_id: Id,
    pub project_name: String,
    pub completed_at: i64,
    pub time: TimeRange,
}

impl CompletedAuditInfo<String> {
    pub fn parse(self) -> CompletedAuditInfo<ObjectId> {
        CompletedAuditInfo {
            audit_id: self.audit_id.parse().unwrap(),
            project_name: self.project_name,
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
            project_name: self.project_name,
            completed_at: self.completed_at,
            time: self.time,
        }
    }

    pub fn stringify_map(map: Vec<Self>) -> Vec<CompletedAuditInfo<String>> {
        map.into_iter().map(|v| v.stringify()).collect()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedbackStars(u8);

impl FeedbackStars {
    pub fn new(value: Option<u8>) -> Result<Option<Self>, &'static str> {
        match value {
            Some(v) if (0..=5).contains(&v) => Ok(Some(FeedbackStars(v))),
            Some(_) => Err("Value must be between 0 and 5"),
            None => Ok(None),
        }
    }

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserFeedbackRating {
    pub quality_of_work: Option<FeedbackStars>,
    pub time_management: Option<FeedbackStars>,
    pub collaboration: Option<FeedbackStars>,
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
