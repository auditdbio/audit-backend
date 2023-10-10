use std::sync::Arc;

use chrono::Utc;
use common::{
    api::{
        auditor::request_auditor,
        chat::{ChatId, CreateMessage, PublicChatId, PublicMessage},
        customer::request_customer,
        events::{EventPayload, PublicEvent},
    },
    context::Context,
    entities::role::Role,
    error,
    services::{EVENTS_SERVICE, PROTOCOL},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::repositories::chat::{ChatRepository, Group};

pub struct ChatService {
    context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicChat {
    pub id: String,
    pub name: String,
    pub members: Vec<PublicChatId>,
    pub last_modified: i64,
    pub last_message: PublicMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preview {
    content: Vec<(PublicChat, PublicMessage)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: ObjectId,
    pub from: ChatId,
    pub chat: ObjectId,
    pub time: i64,
    pub text: String,
}

impl Message {
    pub fn publish(self) -> PublicMessage {
        PublicMessage {
            id: self.id.to_hex(),
            from: self.from.publish(),
            chat: self.chat.to_hex(),
            time: self.time,
            text: self.text,
        }
    }
}

impl ChatService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn send_message(&self, message: CreateMessage) -> error::Result<()> {
        // TODO: check permissions
        let auth = self.context.auth();

        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();

        let message = if let Some(chat) = message.chat {
            Message {
                id: ObjectId::new(),
                from: ChatId {
                    id: *auth.id().unwrap(),
                    role: message.role,
                },
                chat: chat.parse()?,
                time: Utc::now().timestamp_micros(),
                text: message.text,
            }
        } else {
            let stored_message = Message {
                id: ObjectId::new(),
                from: ChatId {
                    id: *auth.id().unwrap(),
                    role: message.role,
                },
                chat: ObjectId::new(),
                time: Utc::now().timestamp_micros(),
                text: message.text,
            };

            repo.create_private(stored_message.clone(), message.to.unwrap().parse()?)
                .await?;
            stored_message
        };

        let chat = repo.message(message.clone()).await?;

        let payload = EventPayload::ChatMessage(message.publish());

        for user_id in chat.members() {
            let event = PublicEvent::new(user_id.id, payload.clone());

            self.context
                .make_request()
                .post(format!(
                    "{}://{}/api/event",
                    PROTOCOL.as_str(),
                    EVENTS_SERVICE.as_str()
                ))
                .json(&event)
                .send()
                .await?;
        }
        Ok(())
    }

    pub async fn preview(&self, role: Role) -> error::Result<Vec<PublicChat>> {
        let auth = self.context.auth();

        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();

        let id = ChatId {
            role,
            id: *auth.id().unwrap(),
        };

        let (chats, privates) = repo.groups_by_user(id).await?;

        let mut chats = chats.into_iter().map(Group::publish).collect::<Vec<_>>();

        for private in privates {
            for id in private.members {
                if &id.id == auth.id().unwrap() {
                    continue;
                }

                let name = if id.role == Role::Auditor {
                    let auditor = request_auditor(&self.context, id.id, auth.clone()).await?;
                    auditor.first_name().clone() + " " + auditor.last_name()
                } else {
                    let customer = request_customer(&self.context, id.id, auth.clone()).await?;
                    customer.first_name + " " + &customer.last_name
                };

                chats.push(PublicChat {
                    id: private.id.to_hex(),
                    name,
                    members: private.members.into_iter().map(ChatId::publish).collect(),
                    last_modified: private.last_modified,
                    last_message: private.last_message.clone().publish(),
                })
            }
        }

        chats.sort_by(|a, b| a.last_modified.cmp(&b.last_modified));

        Ok(chats)
    }

    pub async fn messages(&self, group: ObjectId) -> error::Result<Vec<PublicMessage>> {
        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();
        Ok(repo
            .messages(group)
            .await?
            .into_iter()
            .map(Message::publish)
            .collect())
    }
}
