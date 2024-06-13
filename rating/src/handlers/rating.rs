use actix_web::{
    get, patch, post,
    web::{Path, Json},
};

use common::{
    context::GeneralContext,
    entities::rating::UserFeedback,
    error,
};

use crate::service::rating::{
    CreateFeedback, RatingDetailsResponse,
    RatingService, SummaryResponse,
};

#[get("/rating/auditor/{auditor_id}/")]
pub async fn get_auditor_rating(
    context: GeneralContext,
    auditor_id: Path<String>,
) -> error::Result<Json<SummaryResponse>> {
    Ok(Json(
        RatingService::new(context).get_auditor_rating(auditor_id.parse()?).await?
    ))
}

#[get("/rating/auditor/{auditor_id}/details")]
pub async fn get_auditor_rating_details(
    context: GeneralContext,
    auditor_id: Path<String>,
) -> error::Result<Json<RatingDetailsResponse>> {
    Ok(Json(
        RatingService::new(context).get_auditor_rating_details(auditor_id.parse()?).await?
    ))
}

#[patch("/rating/recalculate/auditor/{auditor_id}")]
pub async fn recalculate_rating(
    context: GeneralContext,
    auditor_id: Path<String>,
) -> error::Result<Json<RatingDetailsResponse>> {
    Ok(Json(
        RatingService::new(context).recalculate_rating(auditor_id.parse()?).await?
    ))
}

#[post("/rating/send_feedback")]
pub async fn send_feedback (
    context: GeneralContext,
    Json(feedback): Json<CreateFeedback>,
) -> error::Result<Json<UserFeedback<String>>> {
    Ok(Json(
        RatingService::new(context).send_feedback(feedback).await?
    ))
}
