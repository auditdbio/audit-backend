use actix_web::{
    get, post,
    web::{Json, Path},
    HttpResponse,
};
use common::{
    api::chat::{CreateMessage, PublicMessage},
    context::Context,
    entities::role::Role,
    error,
};

use crate::services::chat::{ChatService, PublicChat};

#[post("/api/chat/message")]
pub async fn send_message(
    context: Context,
    Json(message): Json<CreateMessage>,
) -> error::Result<HttpResponse> {
    ChatService::new(context).send_message(message).await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/api/chat/preview/{role}")]
pub async fn preview(context: Context, role: Path<Role>) -> error::Result<Json<Vec<PublicChat>>> {
    Ok(Json(
        ChatService::new(context).preview(role.into_inner()).await?,
    ))
}

#[get("/api/chat/{id}")]
pub async fn messages(
    context: Context,
    id: Path<String>,
) -> error::Result<Json<Vec<PublicMessage>>> {
    Ok(Json(
        ChatService::new(context)
            .messages(id.into_inner().parse()?)
            .await?,
    ))
}
