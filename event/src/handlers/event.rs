use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use common::{api::events::PublicEvent, context::GeneralContext, error};
use tokio::sync::Mutex;

use crate::service::event::SessionManager;

#[get("/notifications/{user_id}")]
pub async fn events(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<Mutex<SessionManager>>,
    user_id: web::Path<String>,
) -> error::Result<HttpResponse> {
    crate::service::event::start_session(
        req,
        stream,
        user_id.into_inner().parse()?,
        manager.into_inner(),
    )
    .await
}

#[post("/event")]
pub async fn make_event(
    context: GeneralContext,
    manager: web::Data<Mutex<SessionManager>>,
    Json(event): web::Json<PublicEvent>,
) -> error::Result<HttpResponse> {
    crate::service::event::make_event(context, event, manager.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}
