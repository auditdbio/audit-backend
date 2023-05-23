use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use actix::{Actor, ActorContext, AsyncContext, Handler, Message, Recipient, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws::{self, WsResponseBuilder};
use anyhow::anyhow;
use common::{access_rules::AccessRules, auth::Auth, context::Context, repository::Entity};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{access_rules::SendNotification, repositories::notifications::NotificationsRepository};

#[derive(Message, Clone, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct Notification {
    id: ObjectId,
    user_id: ObjectId,
    inner: NotificationInner,
}

impl Notification {
    pub fn serialize(self) -> PublicNotification {
        PublicNotification {
            id: self.id.to_hex(),
            user_id: self.user_id.to_hex(),
            inner: self.inner,
        }
    }
}

impl Entity for Notification {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicNotification {
    id: String,
    user_id: String,
    inner: NotificationInner,
}

impl From<Notification> for PublicNotification {
    fn from(notification: Notification) -> Self {
        Self {
            id: notification.id.to_hex(),
            user_id: notification.user_id.to_hex(),
            inner: notification.inner,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationInner {
    message: String,
    is_read: bool,
    is_sound: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateNotification {
    pub user_id: ObjectId,
    pub inner: NotificationInner,
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
    initial: Vec<PublicNotification>,
    manager: web::Data<NotificationsManager>,
    auth: bool,
    user_id: ObjectId,
    hb: Instant,
}

impl NotificationsActor {
    pub fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(10) {
                ctx.close(None);
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for NotificationsActor {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NotificationsActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(text)) => {
                let token = text.to_string();
                let auth = Auth::from_token(&token);

                if auth.is_err() || auth.unwrap().id().unwrap() != &self.user_id {
                    return;
                }
                if !self.auth {
                    self.auth = true;
                    ctx.text(serde_json::to_string(&self.initial).unwrap());
                }
            }
            Ok(ws::Message::Pong(_)) => self.hb = Instant::now(),
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            _ => (),
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
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
        if self.auth {
            let msg: PublicNotification = msg.into();
            ctx.text(serde_json::to_string(&msg).unwrap());
        } else {
            self.initial.push(msg.into())
        }
    }
}

pub async fn subscribe_to_notifications(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<NotificationsManager>,
    user_id: ObjectId,
    notifications: &NotificationsRepository,
) -> anyhow::Result<HttpResponse> {
    let session_id = ObjectId::new();
    let mut initial: Vec<PublicNotification> = notifications
        .get_unread(&user_id)
        .await?
        .into_iter()
        .map(|n| n.into())
        .collect();

    initial.reverse();

    let actor = NotificationsActor {
        session_id: session_id.clone(),
        manager: manager.clone(),
        initial,
        auth: false,
        user_id: user_id.clone(),
        hb: Instant::now(),
    };

    let Ok((addr, resp)) = WsResponseBuilder::new(actor, &req, stream).start_with_addr() else{
        return Err(anyhow!("Failed to start websocket"))
    };

    let mut map_lock = manager.subscribers.lock().unwrap();

    let subscribers = map_lock.entry(user_id).or_insert_with(HashMap::new);
    subscribers.insert(session_id, addr.recipient());

    Ok(resp)
}

pub async fn send_notification(
    context: Context,
    manager: web::Data<NotificationsManager>,
    notif: CreateNotification,
    notifications: &NotificationsRepository,
) -> anyhow::Result<()> {
    let notif = Notification {
        id: ObjectId::new(),
        user_id: notif.user_id.clone(),
        inner: notif.inner,
    };
    let auth = context.auth();

    if !SendNotification::get_access(auth, &notif.user_id) {
        return Err(anyhow!("No access to send notification"));
    }

    let map_lock = manager.subscribers.lock().unwrap();

    notifications.insert(&notif).await?;

    if let Some(subscribers) = map_lock.get(&notif.user_id) {
        for (_, recipient) in subscribers {
            recipient.do_send(notif.clone());
        }
    }

    Ok(())
}

pub async fn read(
    context: Context,
    notifications: &NotificationsRepository,
    id: ObjectId,
) -> anyhow::Result<String> {
    let _auth = context.auth();

    notifications.read(id.clone()).await?;

    Ok(id.to_hex())
}
