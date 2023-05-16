use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};

use common::{context::Context, error};

use crate::service::notifications::{
    subscribe_to_notifications, NotificationPayload, NotificationsManager,
};

#[get("/api/notifications")]
pub async fn notifications(
    req: HttpRequest,
    stream: web::Payload,
    context: Context,
    manager: web::Data<NotificationsManager>,
) -> error::Result<HttpResponse> {
    Ok(subscribe_to_notifications(req, stream, context, manager).await?)
}

#[post("/api/send_notification")]
pub async fn send_notification(
    context: Context,
    manager: web::Data<NotificationsManager>,
    Json(send_notification): web::Json<NotificationPayload>,
) -> error::Result<HttpResponse> {
    crate::service::notifications::send_notification(context, manager, send_notification).await?;

    Ok(HttpResponse::Ok().finish())
}
