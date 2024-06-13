use mongodb::bson::{Bson, oid::ObjectId};
use chrono::{Utc, NaiveDateTime, Duration};
use serde::{Deserialize, Serialize};

use common::{
    api::audits::PublicAudit,
    context::GeneralContext,
    error::{self, AddCode},
    entities::{
        audit::PublicAuditStatus,
        rating::{CompletedAuditInfo, Rating, UserFeedback},
        role::Role,
    },
    services::{API_PREFIX, AUDITS_SERVICE, PROTOCOL},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateFeedback {
    pub audit_id: String,
    pub quality_of_work: u8,
    pub time_management: Option<u8>,
    pub collaboration: Option<u8>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummaryResponse {
    pub summary: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RatingDetailsResponse {
    pub id: String,
    pub auditor_id: String,
    pub last_update: i64,
    pub summary: u8,
    pub user_feedbacks: Vec<UserFeedback<String>>,
    pub total_completed_audits: Vec<CompletedAuditInfo<String>>,
    pub completed_last_ninety_days: Vec<CompletedAuditInfo<String>>,
}

impl From<Rating<ObjectId>> for RatingDetailsResponse {
    fn from(rating: Rating<ObjectId>) -> Self {
        let rating = rating.stringify();

        let completed_last_ninety_days = rating
            .total_completed_audits
            .clone()
            .into_iter()
            .filter(|a|
                Utc::now().timestamp_micros() - a.completed_at <= Duration::days(90).num_microseconds().unwrap()
            )
            .collect::<Vec<CompletedAuditInfo<String>>>();

        Self {
            id: rating.id,
            auditor_id: rating.auditor_id,
            last_update: rating.last_update,
            summary: rating.summary,
            user_feedbacks: rating.user_feedbacks,
            total_completed_audits: rating.total_completed_audits,
            completed_last_ninety_days,
        }
    }
}

pub struct RatingService {
    context: GeneralContext,
}

impl RatingService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    async fn create(&self, auditor_id: ObjectId) -> error::Result<Rating<ObjectId>> {
        let mut rating = Rating {
            id: ObjectId::new(),
            auditor_id,
            last_update: Utc::now().timestamp_micros(),
            summary: 0,
            user_feedbacks: vec![],
            total_completed_audits: vec![],
        };

        rating.calculate(&self.context, auditor_id).await?;

        Ok(rating)
    }

    async fn find_or_create(
        &self,
        auditor_id: ObjectId,
        force_update: bool,
    ) -> error::Result<Rating<ObjectId>> {
        let ratings = self.context.try_get_repository::<Rating<ObjectId>>()?;

        let rating = ratings
            .find("auditor_id", &Bson::ObjectId(auditor_id.clone()))
            .await?;

        if rating.is_some() {
            let mut rating = rating.unwrap();
            let last_update_date = NaiveDateTime::from_timestamp_opt(rating.last_update / 1_000_000, 0)
                .expect("Invalid timestamp")
                .date();
            let today_date = Utc::now().date_naive();

            return if last_update_date == today_date && !force_update {
                Ok(rating)
            } else {
                rating.calculate(&self.context, auditor_id).await?;

                ratings.delete("id", &rating.id).await?;
                ratings.insert(&rating).await?;
                Ok(rating)
            }
        }

        let rating = self.create(auditor_id).await?;
        ratings.insert(&rating).await?;

        Ok(rating)
    }

    pub async fn get_auditor_rating(&self, auditor_id: ObjectId) -> error::Result<SummaryResponse> {
        let rating = self.find_or_create(auditor_id, false).await?;
        Ok(SummaryResponse {
            summary: rating.summary,
        })
    }

    pub async fn get_auditor_rating_details(&self, auditor_id: ObjectId) -> error::Result<RatingDetailsResponse> {
        let rating = self.find_or_create(auditor_id, false).await?;
        Ok(RatingDetailsResponse::from(rating))
    }

    pub async fn recalculate_rating(&self, auditor_id: ObjectId) -> error::Result<RatingDetailsResponse> {
        let rating = self.find_or_create(auditor_id, true).await?;
        Ok(RatingDetailsResponse::from(rating))
    }

    pub async fn send_feedback(&self, feedback: CreateFeedback) -> error::Result<UserFeedback<String>> {
        let auth = self.context.auth();
        let user_id = auth.id().unwrap();

        let audit = self
            .context
            .make_request::<PublicAudit>()
            .get(format!(
                "{}://{}/{}/audit/{}",
                PROTOCOL.as_str(),
                AUDITS_SERVICE.as_str(),
                API_PREFIX.as_str(),
                feedback.audit_id,
            ))
            .auth(auth.clone())
            .send()
            .await
            .unwrap()
            .json::<PublicAudit>()
            .await?;

        if audit.status != PublicAuditStatus::Resolved {
            return Err(anyhow::anyhow!("Audit must be resolved").code(403));
        }



        let role = if user_id.to_hex() == audit.auditor_id {
            Role::Auditor
        } else if user_id.to_hex() == audit.customer_id {
            Role::Customer
        } else {
            return Err(anyhow::anyhow!("Unknown user role").code(404));
        };

        Err(anyhow::anyhow!("").code(404))
    }
}
