use actix_web::{
    get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};

use common::{context::Context, error};

use crate::{
    repositories::notifications::NotificationsRepository,
    service::notifications::{
        read, subscribe_to_notifications, CreateNotification, NotificationsManager,
    },
};

#[get("/api/notifications")]
pub async fn notifications(
    req: HttpRequest,
    stream: web::Payload,
    context: Context,
    manager: web::Data<NotificationsManager>,
    notifications: web::Data<NotificationsRepository>,
) -> error::Result<HttpResponse> {
    Ok(subscribe_to_notifications(req, stream, context, manager, &notifications).await?)
}

#[post("/api/send_notification")]
pub async fn send_notification(
    context: Context,
    manager: web::Data<NotificationsManager>,
    Json(new_notification): web::Json<CreateNotification>,
    notifs: web::Data<NotificationsRepository>,
) -> error::Result<HttpResponse> {
    crate::service::notifications::send_notification(context, manager, new_notification, &notifs)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[patch("/api/read_notification/{id}")]
pub async fn read_notification(
    context: Context,
    notifs: web::Data<NotificationsRepository>,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    read(context, &notifs, id.parse()?).await?;

    Ok(HttpResponse::Ok().finish())
}
