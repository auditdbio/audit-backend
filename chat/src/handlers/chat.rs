use actix_web::{
    get, post,
    web::{Json, Path},
    HttpResponse,
};
use common::{api::chat::PublicMessage, context::Context, error};

use crate::services::chat::{ChatService, Preview};

#[post("/api/chat/message")]
pub async fn send_message(
    context: Context,
    Json(message): Json<PublicMessage>,
) -> error::Result<HttpResponse> {
    ChatService::new(context).send_message(message).await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/api/chat/preview")]
pub async fn preview(context: Context) -> error::Result<Json<Preview>> {
    Ok(Json(ChatService::new(context).preview().await?))
}

#[get("api/chat/{id}")]
pub async fn messages(context: Context, id: Path<String>) -> error::Result<Json<Vec<String>>> {
    Ok(Json(
        ChatService::new(context)
            .messages(id.into_inner().parse()?)
            .await?,
    ))
}
