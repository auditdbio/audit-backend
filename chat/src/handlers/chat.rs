use crate::repositories::chat::Chat;
use actix_web::{
    get, patch, post,
    web::{Json, Path},
    HttpResponse,
};
use common::{
    api::chat::{CreateMessage, PublicMessage},
    context::GeneralContext,
    entities::role::Role,
    error,
};

use crate::services::chat::{ChatService, PublicChat};

#[post("/api/chat/message")]
pub async fn send_message(
    context: GeneralContext,
    Json(message): Json<CreateMessage>,
) -> error::Result<Json<Chat>> {
    Ok(Json(ChatService::new(context).send_message(message).await?))
}

#[get("/api/chat/preview/{role}")]
pub async fn preview(
    context: GeneralContext,
    role: Path<Role>,
) -> error::Result<Json<Vec<PublicChat>>> {
    Ok(Json(
        ChatService::new(context).preview(role.into_inner()).await?,
    ))
}

#[get("/api/chat/{id}")]
pub async fn messages(
    context: GeneralContext,
    id: Path<String>,
) -> error::Result<Json<Vec<PublicMessage>>> {
    Ok(Json(
        ChatService::new(context)
            .messages(id.into_inner().parse()?)
            .await?,
    ))
}

#[patch("/api/chat/{id}/unread/{unread}")]
pub async fn chat_unread(
    context: GeneralContext,
    params: Path<(String, i32)>,
) -> error::Result<HttpResponse> {
    ChatService::new(context)
        .unread_messages(params.0.parse()?, params.1)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
