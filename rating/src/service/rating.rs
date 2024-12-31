use mongodb::bson::{Bson, doc, oid::ObjectId};
use chrono::{Utc, NaiveDateTime, Duration};
use serde::{Deserialize, Serialize};

use common::{
    api::audits::PublicAudit,
    context::GeneralContext,
    error::{self, AddCode},
    entities::{
        audit::PublicAuditStatus,
        rating::{
            CompletedAuditInfo, Rating,
            RoleRating, UserFeedback,
            FeedbackFrom, UserFeedbackRating,
            FeedbackStars, PublicUserFeedback,
        },
        role::Role,
    },
    services::{API_PREFIX, AUDITS_SERVICE, PROTOCOL},
};


pub struct RatingService {
    context: GeneralContext,
}

impl RatingService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    async fn create(&self, user_id: ObjectId, role: Role) -> error::Result<Rating<ObjectId>> {
        let role_rating = RoleRating {
            last_update: Utc::now().timestamp_micros(),
            summary: 0.0,
            rating_details: None,
            user_feedbacks: vec![],
            total_completed_audits: vec![],
        };

        let rating = Rating {
            id: ObjectId::new(),
            user_id,
            auditor: role_rating.clone(),
            customer: role_rating,
            last_modified: Utc::now().timestamp_micros(),
        };

        let rating = rating.calculate(&self.context, role).await?;

        Ok(rating)
    }

    async fn find_or_create(
        &self,
        user_id: ObjectId,
        role: Role,
        force_update: bool,
        recalculate: bool,
    ) -> error::Result<Rating<ObjectId>> {
        let ratings = self.context.try_get_repository::<Rating<ObjectId>>()?;
        let rating = ratings
            .find("user_id", &Bson::ObjectId(user_id.clone()))
            .await?;

        if rating.is_some() {
            let rating = rating.unwrap();
            let role_rating = if role == Role::Auditor {
                rating.auditor.clone()
            } else {
                rating.customer.clone()
            };

            let last_update_date = NaiveDateTime::from_timestamp_opt(role_rating.last_update / 1_000_000, 0)
                .expect("Invalid timestamp")
                .date();
            let today_date = Utc::now().date_naive();

            return if last_update_date == today_date && !force_update {
                Ok(rating)
            } else if !recalculate {
                Ok(rating)
            } else {
                let rating = rating.calculate(&self.context, role).await?;

                // ratings.update_one(doc! {"id": &rating.id}, &rating).await?;
                ratings.delete("id", &rating.id).await?;
                ratings.insert(&rating).await?;
                Ok(rating)
            }
        }

        let rating = self.create(user_id, role).await?;
        ratings.insert(&rating).await?;

        Ok(rating)
    }

    pub async fn get_user_rating(&self, user_id: ObjectId, role: Role) -> error::Result<SummaryResponse> {
        let rating = self.find_or_create(user_id, role, false, true).await?;
        let summary = if role == Role::Auditor {
            rating.auditor.summary
        } else {
            rating.customer.summary
        };

        Ok(SummaryResponse {
            summary,
        })
    }

    pub async fn get_user_rating_details(&self, user_id: ObjectId, role: Role) -> error::Result<RatingDetailsResponse> {
        let rating = self.find_or_create(user_id, role, false, true).await?;
        Ok(RatingDetailsResponse::from_rating(&self.context, rating, role, 90).await?)
    }

    pub async fn recalculate_rating(&self, user_id: ObjectId, role: Role) -> error::Result<RatingDetailsResponse> {
        let rating = self.find_or_create(user_id, role, true, true).await?;
        Ok(RatingDetailsResponse::from_rating(&self.context, rating, role, 90).await?)
    }

    pub async fn send_feedback(&self, feedback: CreateFeedback) -> error::Result<UserFeedback<String>> {
        let auth = self.context.auth();
        let user_id = auth.id().unwrap();

        if feedback.quality_of_work.is_none()
            && feedback.time_management.is_none()
            && feedback.collaboration.is_none() {
            return Err(anyhow::anyhow!("At least one assessment required").code(400));
        }

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

        if audit.no_customer {
            return Err(anyhow::anyhow!("Not available for audits without a customer").code(403));
        }

        let (current_role, receiver_role, receiver_id) = if user_id.to_hex() == audit.auditor_id {
            (Role::Auditor, Role::Customer, audit.customer_id)
        } else if user_id.to_hex() == audit.customer_id {
            (Role::Customer, Role::Auditor, audit.auditor_id)
        } else {
            return Err(anyhow::anyhow!("Unknown user role").code(404));
        };

        let create_feedback = UserFeedback {
            id: ObjectId::new(),
            audit_id: feedback.audit_id.parse().unwrap(),
            from: FeedbackFrom {
                user_id,
                role: current_role,
            },
            rating: UserFeedbackRating {
                quality_of_work: FeedbackStars::new(feedback.quality_of_work).unwrap(),
                time_management: FeedbackStars::new(feedback.time_management).unwrap(),
                collaboration: FeedbackStars::new(feedback.collaboration).unwrap(),
            },
            created_at: Utc::now().timestamp_micros(),
            comment: feedback.comment,
        };

        let mut rating = self.find_or_create(
            receiver_id.parse().unwrap(),
            current_role,
            false,
            false,
        ).await?;

        let role_rating = if receiver_role == Role::Auditor {
            &mut rating.auditor
        } else {
            &mut rating.customer
        };

        if let Some(existing_feedback) = role_rating
            .user_feedbacks
            .iter_mut()
            .find(|fb| {
            fb.audit_id == create_feedback.audit_id && fb.from.user_id == create_feedback.from.user_id
        }) {
            *existing_feedback = create_feedback.clone();
        } else {
            role_rating.user_feedbacks.push(create_feedback.clone());
        }

        let rating = rating.calculate(&self.context, receiver_role).await?;

        let ratings = self.context.try_get_repository::<Rating<ObjectId>>()?;
        ratings.delete("id", &rating.id).await?;
        ratings.insert(&rating).await?;
        // ratings.update_one(doc! {"id": &rating.id}, &rating).await?;

        Ok(create_feedback.stringify())
    }

    pub async fn get_feedback(
        &self,
        receiver_id: ObjectId,
        audit_id: ObjectId,
        role: Role,
    ) -> error::Result<UserFeedback<String>> {
        let _auth = self.context.auth();

        let rating = self.find_or_create(
            receiver_id,
            role,
            false,
            true,
        ).await?;

        let feedbacks = if role == Role::Auditor {
            rating.auditor.user_feedbacks
        } else {
            rating.customer.user_feedbacks
        };

        if let Some(feedback) = feedbacks
            .iter()
            .find(|fb| fb.audit_id == audit_id) {
            Ok(feedback.clone().stringify())
        } else {
            Err(anyhow::anyhow!("Feedback not found").code(204))
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateFeedback {
    pub audit_id: String,
    pub quality_of_work: Option<u8>,
    pub time_management: Option<u8>,
    pub collaboration: Option<u8>,
    pub comment: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummaryResponse {
    pub summary: f32,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RatingDetailsResponse {
    pub id: String,
    pub user_id: String,
    pub role: Role,
    pub last_update: i64,
    pub summary: f32,
    pub rating_details: Option<String>,
    pub user_feedbacks: Vec<PublicUserFeedback>,
    pub total_completed_audits: usize,
    pub completed_last_ninety_days: Vec<CompletedAuditInfo<String>>,
}

impl RatingDetailsResponse {
    pub async fn from_rating(
        context: &GeneralContext,
        rating: Rating<ObjectId>,
        role: Role,
        days: i64
    ) -> error::Result<RatingDetailsResponse> {
        let rating = rating.stringify();

        let role_rating = if role == Role::Auditor {
            rating.auditor
        } else {
            rating.customer
        };

        let completed_last_ninety_days = role_rating
            .total_completed_audits
            .clone()
            .into_iter()
            .filter(|a|
                Utc::now().timestamp_micros() - a.completed_at <= Duration::days(days).num_microseconds().unwrap()
            )
            .collect::<Vec<CompletedAuditInfo<String>>>();

        let mut user_feedbacks = vec![];
        for feedback in role_rating.user_feedbacks {
            user_feedbacks.push(PublicUserFeedback::new(context, feedback.parse()).await?);
        }

        Ok(RatingDetailsResponse {
            id: rating.id,
            user_id: rating.user_id,
            role,
            last_update: role_rating.last_update,
            summary: role_rating.summary,
            rating_details: role_rating.rating_details,
            user_feedbacks,
            total_completed_audits: role_rating.total_completed_audits.len(),
            completed_last_ninety_days,
        })
    }
}
