use actix_web::{
    get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use common::{context::GeneralContext, entities::notification::CreateNotification, error};
use mongodb::bson::doc;

use crate::{
    repositories::notifications::NotificationsRepository,
    service::notifications::{get_unread_notifications, read, PublicNotification},
};

#[post("/api/send_notification")]
pub async fn send_notification(
    context: GeneralContext,
    Json(new_notification): web::Json<CreateNotification>,
    notifs: web::Data<NotificationsRepository>,
) -> error::Result<HttpResponse> {
    crate::service::notifications::send_notification(context, new_notification, &notifs).await?;

    Ok(HttpResponse::Ok().finish())
}

#[patch("/api/read_notification/{id}")]
pub async fn read_notification(
    context: GeneralContext,
    notifs: web::Data<NotificationsRepository>,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let id = read(context, &notifs, id.parse()?).await?;

    Ok(HttpResponse::Ok().json(doc! {"id": id}))
}

#[get("/api/unread_notifications")]
pub async fn unread_notifications(
    context: GeneralContext,
    notifs: web::Data<NotificationsRepository>,
) -> error::Result<Json<Vec<PublicNotification>>> {
    Ok(Json(get_unread_notifications(context, &notifs).await?))
}
