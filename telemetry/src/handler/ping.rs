use actix_web::{
    get,
    web::{self, Json},
    HttpResponse, ResponseError,
};
use common::{
    api::events::{post_event, EventPayload, PublicEvent},
    context::GeneralContext,
};
use mongodb::bson::oid::ObjectId;

use crate::service::ping::{self, Service, Status};

#[get("/status")]
pub async fn status(context: GeneralContext, services: web::Data<Vec<Service>>) -> Json<Status> {
    let status = ping::status(context, &services).await;
    Json(status)
}

#[get("/telemetry/update")]
pub async fn update(context: GeneralContext) -> HttpResponse {
    match ping::update(&context).await {
        Ok(_) => {
            let event = PublicEvent::new(ObjectId::new(), None, EventPayload::VersionUpdate);
            post_event(&context, event, context.server_auth())
                .await
                .unwrap();
            HttpResponse::Ok().finish()
        }
        Err(err) => err.error_response(),
    }
}
