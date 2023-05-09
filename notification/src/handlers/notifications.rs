use actix::{Actor, StreamHandler};
use actix_web::{get, HttpRequest, web, HttpResponse};
use actix_web_actors::ws;
use common::error;
use anyhow::anyhow;
use ws::Message;

pub struct Notification {
    message: String,
}

struct NotificationsActor;

impl Actor for NotificationsActor {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NotificationsActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Text(text)) => ctx.text(text),
            Ok(Message::Close(x)) => {
                
                ctx.close(x);
            }
            _ => (),
        }
    }
}

#[get("/api/notifications")]
async fn index(req: HttpRequest, stream: web::Payload) -> error::Result<HttpResponse> {
    let Ok(resp)  = ws::start(NotificationsActor, &req, stream) else {
        return Err(anyhow!("Failed to start websocket").into());
    };
    Ok(resp)
}