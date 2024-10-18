use std::sync::Arc;

use chrono::Utc;
use common::{
    api::{
        auditor::request_auditor,
        chat::{ChatId, CreateMessage, MessageKind, PublicReadId, PublicMessage, PublicChat},
        customer::request_customer,
        events::{EventPayload, PublicEvent},
    },
    context::GeneralContext,
    entities::role::Role,
    error::{self, AddCode},
    services::{API_PREFIX, EVENTS_SERVICE, PROTOCOL},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::repositories::chat::{ChatRepository, Group, ReadId};

pub struct ChatService {
    context: GeneralContext,
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
    pub kind: Option<MessageKind>,
}

impl Message {
    pub fn publish(self) -> PublicMessage {
        PublicMessage {
            id: self.id.to_hex(),
            from: self.from.publish(),
            chat: self.chat.to_hex(),
            time: self.time,
            text: self.text,
            kind: self.kind,
        }
    }
}

impl ChatService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn send_message(&self, message: CreateMessage) -> error::Result<PublicChat> {
        // TODO: check permissions
        let auth = self.context.auth();

        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();

        let from = ChatId {
            id: auth.id().unwrap(),
            role: message.role,
        };

        let message = if let Some(chat) = message.chat {
            Message {
                id: ObjectId::new(),
                from,
                chat: chat.parse()?,
                time: Utc::now().timestamp_micros(),
                text: message.text,
                kind: message.kind,
            }
        } else {
            let existing_chat = repo
                .find_by_members(vec![
                    from.clone(),
                    message.to.clone().unwrap().parse()?
                ])
                .await?;

            if let Some(existing_private) = existing_chat {
                Message {
                    id: ObjectId::new(),
                    from,
                    chat: existing_private.id,
                    time: Utc::now().timestamp_micros(),
                    text: message.text,
                    kind: message.kind,
                }
            } else {
                let stored_message = Message {
                    id: ObjectId::new(),
                    from,
                    chat: ObjectId::new(),
                    time: Utc::now().timestamp_micros(),
                    text: message.text,
                    kind: message.kind,
                };

                repo.create_private(stored_message.clone(), message.to.unwrap().parse()?)
                    .await?;
                stored_message
            }
        };

        let chat = repo.message(message.clone()).await?;

        let payload = EventPayload::ChatMessage(message.publish());

        for user_id in chat.members() {
            if user_id.id != auth.id().unwrap() {
                repo.unread(chat.chat_id(), user_id.id, None).await?;
            }

            let event = PublicEvent::new(user_id.id, None, payload.clone());

            self.context
                .make_request()
                .post(format!(
                    "{}://{}/{}/event",
                    PROTOCOL.as_str(),
                    EVENTS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                ))
                .json(&event)
                .send()
                .await?;
        }
        Ok(chat.publish())
    }

    pub async fn preview(&self, role: Role) -> error::Result<Vec<PublicChat>> {
        let auth = self.context.auth();

        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();

        let id = ChatId {
            role,
            id: auth.id().unwrap(),
        };

        let (chats, privates) = repo.groups_by_user(id).await?;

        let mut chats = chats.into_iter().map(Group::publish).collect::<Vec<_>>();

        for private in privates {
            for id in private.members {
                if id.id == auth.id().unwrap() {
                    continue;
                }

                let (name, avatar) = if id.role == Role::Auditor {
                    let auditor = match request_auditor(&self.context, id.id, auth.clone()).await {
                        Ok(auditor) => auditor,
                        _ => continue
                    };
                    if auditor.is_empty() {
                        continue;
                    }
                    (
                        auditor.first_name().clone() + " " + auditor.last_name(),
                        auditor.avatar().to_string(),
                    )
                } else {
                    let customer = match request_customer(&self.context, id.id, auth.clone()).await {
                        Ok(customer) => customer,
                        _ => continue
                    };
                    if customer.user_id.is_empty() {
                        continue;
                    }
                    (
                        customer.first_name + " " + &customer.last_name,
                        customer.avatar,
                    )
                };

                let unread = if let Some(unread) = private.unread.clone() {
                    unread.clone().into_iter().map(ReadId::publish).collect()
                } else {
                    let mut unread = Vec::new();
                    for member in &private.members {
                        unread.push(PublicReadId {
                            id: member.id.to_hex(),
                            unread: 0,
                        });
                    }
                    unread
                };

                chats.push(PublicChat {
                    id: private.id.to_hex(),
                    name,
                    avatar: Some(avatar),
                    members: private.members.into_iter().map(ChatId::publish).collect(),
                    last_modified: private.last_modified,
                    last_message: private.last_message.clone().publish(),
                    unread,
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

    pub async fn unread_messages(&self, group: ObjectId, unread: i32) -> error::Result<()> {
        let auth = self.context.auth();
        let user_id = auth.id().unwrap();

        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();

        repo.unread(group, user_id, Some(unread)).await?;
        Ok(())
    }

    pub async fn delete_message(&self, chat_id: ObjectId, message_id: ObjectId) -> error::Result<()> {
        let auth = self.context.auth();
        let id = auth.id().unwrap();

        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();

        let chat = repo.find(chat_id).await?;
        let chat_members = chat.members();

        if !chat_members.iter().any(|member| member.id == id) {
            return Err(anyhow::anyhow!("User is not available to delete this message").code(403));
        }

        let payload = EventPayload::ChatDeleteMessage(message_id.to_hex());

        for member in chat_members {
            let event = PublicEvent::new(member.id, None, payload.clone());
            self.context
                .make_request()
                .post(format!(
                    "{}://{}/{}/event",
                    PROTOCOL.as_str(),
                    EVENTS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                ))
                .json(&event)
                .send()
                .await?;
        }

        repo.delete_message(chat_id, message_id).await?;

        Ok(())
    }
}
