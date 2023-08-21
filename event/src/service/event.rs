use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use actix::{Actor, ActorContext, AsyncContext, Handler, Message, Recipient, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws::{self, WsResponseBuilder};
use common::{
    api::events::PublicEvent,
    auth::Auth,
    context::Context,
    error::{self, AddCode},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Message, Clone, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct Event {
    inner: PublicEvent,
}

#[derive(Clone)]
pub struct Session {
    session_id: ObjectId,
    user_id: ObjectId,
    manager: Arc<Mutex<SessionManager>>,
    auth: bool,
    hb: Instant,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;
}

impl Session {
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

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(text)) => {
                let token = text.to_string();
                let Ok(Some(auth)) = Auth::from_token(&token) else {
                    log::warn!("unsuccessful auth");
                    return;
                };

                log::info!("successful auth");

                if auth.id().unwrap() != &self.user_id {
                    log::warn!("unsuccessful auth");
                    return;
                }

                if !self.auth {
                    log::info!("auth is true");

                    self.auth = true;
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
        let mut lock = self.manager.blocking_lock();
        lock.remove(&self.session_id, &self.user_id);
    }
}

impl Handler<Event> for Session {
    type Result = ();

    fn handle(&mut self, msg: Event, ctx: &mut Self::Context) {
        if self.auth {
            ctx.text(serde_json::to_string(&msg.inner).unwrap());
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct UserSessions {
    sessions: HashMap<ObjectId, Recipient<Event>>,
}

#[derive(Clone, Default)]
pub struct SessionManager {
    users: HashMap<ObjectId, UserSessions>,
}

impl SessionManager {
    pub fn remove(&mut self, session_id: &ObjectId, user_id: &ObjectId) {
        if let Some(user) = self.users.get_mut(user_id) {
            user.sessions.remove(session_id);
        }
    }
}

pub async fn start_session(
    req: HttpRequest,
    stream: web::Payload,
    user_id: ObjectId,
    manager: Arc<Mutex<SessionManager>>,
) -> error::Result<HttpResponse> {
    let session_id = ObjectId::new();
    let session = Session {
        session_id,
        user_id,
        manager: manager.clone(),
        auth: false,
        hb: Instant::now(),
    };

    let Ok((addr, resp)) = WsResponseBuilder::new(session, &req, stream).start_with_addr() else{
        return Err(anyhow::anyhow!("Failed to start websocket").code(500))
    };

    let mut lock = manager.lock().await;

    if let Some(user) = lock.users.get_mut(&user_id) {
        user.sessions.insert(session_id, addr.recipient());
    } else {
        let mut sessions = UserSessions {
            sessions: HashMap::new(),
        };
        sessions.sessions.insert(session_id, addr.recipient());
        lock.users.insert(user_id, sessions);
    }
    Ok(resp)
}

pub async fn make_event(
    _context: Context,
    event: PublicEvent,
    manager: Arc<Mutex<SessionManager>>,
) -> error::Result<()> {
    // TODO: make auth
    let lock = manager.lock().await;

    if event.payload.for_all() {
        for user in lock.users.values() {
            for (_, addr) in user.sessions.iter() {
                addr.do_send(Event {
                    inner: event.clone(),
                });
            }
        }
    } else if let Some(user) = lock.users.get(&event.user_id) {
        let event = Event { inner: event };
        for (_, addr) in user.sessions.iter() {
            addr.do_send(event.clone());
        }
    }

    Ok(())
}
