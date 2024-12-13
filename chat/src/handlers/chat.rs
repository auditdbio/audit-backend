use std::collections::HashMap;
use actix_web::{
    get, patch, post, delete,
    web::{Json, Path, Query},
    HttpResponse,
};
use common::{
    api::chat::{CreateMessage, PublicMessage, PublicChat},
    context::GeneralContext,
    entities::role::ChatRole,
    error,
};
use common::api::chat::ChangeUnread;

use crate::services::chat::{ChatService};

#[post("/chat/message")]
pub async fn send_message(
    context: GeneralContext,
    Json(message): Json<CreateMessage>,
) -> error::Result<Json<PublicChat>> {
    Ok(Json(ChatService::new(context).send_message(message).await?))
}

#[get("/chat/preview/{role}")]
pub async fn preview(
    context: GeneralContext,
    role: Path<ChatRole>,
    query: Query<HashMap<String, String>>,
) -> error::Result<Json<Vec<PublicChat>>> {
    let org_id = query.get("org_id");
    Ok(Json(
        ChatService::new(context).preview(role.into_inner(), org_id).await?,
    ))
}

#[get("/chat/{id}")]
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

#[patch("/chat/{id}/unread/{unread}")]
pub async fn chat_unread(
    context: GeneralContext,
    params: Path<(String, i32)>,
    Json(data): Json<ChangeUnread>,
) -> error::Result<HttpResponse> {
    ChatService::new(context)
        .unread_messages(params.0.parse()?, params.1, data)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[delete("/chat/{chat_id}/message/{message_id}")]
pub async fn delete_message(
    context: GeneralContext,
    params: Path<(String, String)>,
) -> error::Result<HttpResponse> {
    ChatService::new(context)
        .delete_message(params.0.parse()?, params.1.parse()?)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
