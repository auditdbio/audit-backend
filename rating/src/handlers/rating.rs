use actix_web::{
    get, patch, post,
    web::{Path, Json},
};

use common::{
    context::GeneralContext,
    entities::{
        rating::UserFeedback,
        role::Role,
    },
    error,
};
use crate::service::rating::{
    CreateFeedback, RatingDetailsResponse,
    RatingService, SummaryResponse,
};

#[get("/rating/{role}/{user_id}")]
pub async fn get_user_rating(
    context: GeneralContext,
    path: Path<(Role, String)>,
) -> error::Result<Json<SummaryResponse>> {
    let (role, user_id) = path.into_inner();
    Ok(Json(
        RatingService::new(context).get_user_rating(user_id.parse()?, role).await?
    ))
}

#[get("/rating/{role}/{user_id}/details")]
pub async fn get_user_rating_details(
    context: GeneralContext,
    path: Path<(Role, String)>,
) -> error::Result<Json<RatingDetailsResponse>> {
    let (role, user_id) = path.into_inner();
    Ok(Json(
        RatingService::new(context).get_user_rating_details(user_id.parse()?, role).await?
    ))
}

#[patch("/rating/recalculate/{role}/{user_id}")]
pub async fn recalculate_rating(
    context: GeneralContext,
    path: Path<(Role, String)>,
) -> error::Result<Json<RatingDetailsResponse>> {
    let (role, user_id) = path.into_inner();
    Ok(Json(
        RatingService::new(context).recalculate_rating(user_id.parse()?, role).await?
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
