use std::{collections::HashMap, sync::Mutex};

use actix::{Actor, ActorContext, Handler, Message, Recipient, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws::{self, WsResponseBuilder};
use anyhow::anyhow;
use common::{
    access_rules::{AccessRules},
    context::Context,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::access_rules::SendNotification;

#[derive(Message, Clone, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct Notification {
    message: String,
}

pub struct NotificationsManager {
    subscribers: Mutex<HashMap<ObjectId, HashMap<ObjectId, Recipient<Notification>>>>,
}

impl NotificationsManager {
    pub fn new() -> Self {
        Self {
            subscribers: Mutex::new(HashMap::new()),
        }
    }
}

struct NotificationsActor {
    session_id: ObjectId,
    manager: web::Data<NotificationsManager>,
}

impl Actor for NotificationsActor {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NotificationsActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Close(_)) => ctx.stop(),
            _ => (),
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        self.manager
            .subscribers
            .lock()
            .unwrap()
            .remove(&self.session_id);
    }
}

impl Handler<Notification> for NotificationsActor {
    type Result = ();

    fn handle(&mut self, msg: Notification, ctx: &mut Self::Context) {
        ctx.text(msg.message);
    }
}

pub async fn subscribe_to_notifications(
    req: HttpRequest,
    stream: web::Payload,
    context: Context,
    manager: web::Data<NotificationsManager>,
) -> anyhow::Result<HttpResponse> {
    let user_id = context.auth().id().unwrap().clone();
    let session_id = ObjectId::new();

    let Ok((addr, resp)) = WsResponseBuilder::new(NotificationsActor {session_id: session_id.clone(), manager: manager.clone()}, &req, stream).start_with_addr() else{
        return Err(anyhow!("Failed to start websocket"))
    };

    let mut map_lock = manager.subscribers.lock().unwrap();

    let subscribers = map_lock.entry(user_id).or_insert_with(HashMap::new);
    subscribers.insert(session_id, addr.recipient());

    Ok(resp)
}

#[derive(Deserialize, Debug)]
pub struct NotificationPayload {
    pub user_id: String,
    pub notification: Notification,
}

pub async fn send_notification(
    context: Context,
    manager: web::Data<NotificationsManager>,
    send_notification: NotificationPayload,
) -> anyhow::Result<()> {
    let user_id = send_notification.user_id.parse()?;
    let auth = context.auth();

    if SendNotification::get_access(auth, &user_id) {
        return Err(anyhow!("No access to send notification"));
    }

    let map_lock = manager.subscribers.lock().unwrap();

    if let Some(subscribers) = map_lock.get(&user_id) {
        for (_, recipient) in subscribers {
            recipient.do_send(send_notification.notification.clone());
        }
    }

    Ok(())
}
