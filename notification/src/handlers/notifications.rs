use actix_web::{
    get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};

use common::{context::Context, entities::notification::CreateNotification, error};
use mongodb::bson::doc;

use crate::{
    repositories::notifications::NotificationsRepository,
    service::notifications::{
        get_unread_notifications, read, subscribe_to_notifications, NotificationsManager,
        PublicNotification,
    },
};

#[get("/api/notifications/{user_id}")]
pub async fn notifications(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<NotificationsManager>,
    notifications: web::Data<NotificationsRepository>,
    user_id: web::Path<String>,
) -> error::Result<HttpResponse> {
    subscribe_to_notifications(req, stream, manager, user_id.parse()?, &notifications).await
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
    let id = read(context, &notifs, id.parse()?).await?;

    Ok(HttpResponse::Ok().json(doc! {"id": id}))
}

#[get("/api/unread_notifications")]
pub async fn unread_notifications(
    context: Context,
    notifs: web::Data<NotificationsRepository>,
) -> error::Result<Json<Vec<PublicNotification>>> {
    Ok(Json(get_unread_notifications(context, &notifs).await?))
}
