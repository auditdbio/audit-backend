use std::sync::Arc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use chrono::Utc;
use common::{
    api::{
        auditor::request_auditor,
        chat::{ChatId, CreateMessage, ChangeUnread, MessageKind, PublicReadId, PublicMessage, PublicChat},
        customer::request_customer,
        events::{EventPayload, PublicEvent, post_event},
        organization::{get_organization, get_my_organizations, GetOrganizationQuery},
    },
    context::GeneralContext,
    entities::{
        organization::OrganizationMember,
        role::ChatRole,
    },
    error::{self, AddCode},
    services::{API_PREFIX, EVENTS_SERVICE, PROTOCOL, USERS_SERVICE},
};

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
        let current_id = auth.id().unwrap();

        let mut is_new_chat = false;

        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();

        let from = if message.role == ChatRole::Organization {
            let org_id = message
                .from_org_id
                .ok_or_else(|| anyhow::anyhow!("Field 'from_org_id' is required for 'Organization' role").code(400))?
                .parse()?;

            let org_user_response = self
                .context
                .make_request::<OrganizationMember>()
                .auth(auth)
                .get(format!(
                    "{}://{}/{}/organization/{}/members/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    org_id,
                    current_id,
                ))
                .send()
                .await?;

            if org_user_response.status().is_success() {
                ChatId {
                    id: org_id,
                    role: message.role,
                    org_user_id: Some(current_id.clone()),
                }
            } else {
                return Err(anyhow::anyhow!("The user is not a member of the specified organization").code(400))
            }
        } else {
            ChatId {
                id: current_id.clone(),
                role: message.role,
                org_user_id: None,
            }
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
                is_new_chat = true;
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
        let public_chat = chat.publish();

        let message_payload = EventPayload::ChatMessage(message.publish());
        let new_chat_payload = EventPayload::NewChat(public_chat.clone());

        for member in chat.members() {
            if member.id != current_id {
                repo.unread(chat.chat_id(), member.id, None).await?;
            }

            if member.role == ChatRole::Organization {
                let org_members = get_organization(&self.context, member.id, None)
                    .await?
                    .members
                    .unwrap_or(vec![]);

                for org_member in org_members {
                    let org_user_id = org_member.user_id.parse()?;
                    let message_event = PublicEvent::new(
                        org_user_id,
                        None,
                        message_payload.clone(),
                    );

                    post_event(&self.context, message_event, auth).await?;

                    if is_new_chat && org_member.user_id != current_id.to_hex() {
                        let new_chat_event = PublicEvent::new(
                            org_user_id,
                            None,
                            new_chat_payload.clone(),
                        );
                        post_event(&self.context, new_chat_event, auth).await?;
                    }
                }
            } else {
                let message_event = PublicEvent::new(
                    member.id,
                    None,
                    message_payload.clone(),
                );
                post_event(&self.context, message_event, auth).await?;

                if is_new_chat && member.id != current_id {
                    let new_chat_event = PublicEvent::new(
                        member.id,
                        None,
                        new_chat_payload.clone(),
                    );
                    post_event(&self.context, new_chat_event, auth).await?;
                }
            }
        }

        Ok(public_chat)
    }

    pub async fn preview(&self, role: ChatRole, org_id: Option<&String>) -> error::Result<Vec<PublicChat>> {
        let auth = self.context.auth();
        let current_id = auth.id().unwrap();

        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();

        let chat_id = ChatId {
            role,
            id: org_id.map_or(current_id, |org_id| org_id.parse().unwrap()),
            org_user_id: None,
        };

        let my_organizations = get_my_organizations(&self.context).await?;
        let mut my_org_ids = my_organizations
            .owner
            .iter()
            .map(|o| o.id.clone())
            .collect::<Vec<String>>();

        my_org_ids.extend(my_organizations
            .member
            .iter()
            .map(|o| o.id.clone())
            .collect::<Vec<String>>()
        );

        let (chats, privates) = repo.groups_by_user(chat_id).await?;
        let mut chats = chats.into_iter().map(Group::publish).collect::<Vec<_>>();

        for private in privates {
            for member in private.members {
                if member.id == current_id {
                    continue;
                }

                if role == ChatRole::Organization {
                    if member.org_user_id == Some(current_id) {
                        continue;
                    } else if member.org_user_id.is_none() && my_org_ids.contains(&member.id.to_hex()) {
                        continue;
                    }
                }

                let (name, avatar) = if member.role == ChatRole::Auditor {
                    let auditor = match request_auditor(&self.context, member.id, auth.clone()).await {
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
                } else if member.role == ChatRole::Customer {
                    let customer = match request_customer(&self.context, member.id, auth.clone()).await {
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
                } else {
                    let query = GetOrganizationQuery {
                        with_members: Some(false),
                    };
                    let organization = match get_organization(&self.context, member.id, Some(query)).await {
                        Ok(org) => org,
                        _ => continue
                    };
                    (organization.name, organization.avatar.unwrap_or("".to_string()))
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
                    creator: private.creator.map(ChatId::publish),
                })
            }
        }

        chats.sort_by(|a, b| a.last_modified.cmp(&b.last_modified));

        Ok(chats)
    }

    pub async fn messages(&self, group: ObjectId) -> error::Result<Vec<PublicMessage>> {
        let auth = self.context.auth();
        let current_id = auth.id().unwrap();

        let repo = self
            .context
            .get_repository_manual::<Arc<ChatRepository>>()
            .unwrap();

        let chat = repo.find(group).await?;
        let chat_members = chat.members();

        for member in chat_members {
            if member.id == current_id || member.org_user_id == Some(current_id) {
                let messages = repo
                    .messages(group)
                    .await?
                    .into_iter()
                    .map(Message::publish)
                    .collect();
                return Ok(messages)
            } else if member.role == ChatRole::Organization {
                let org_members = get_organization(&self.context, member.id, None)
                    .await?
                    .members
                    .unwrap_or(vec![])
                    .into_iter()
                    .map(|m| m.user_id)
                    .collect::<Vec<String>>();

                if org_members.contains(&current_id.to_hex()) {
                    let messages = repo
                        .messages(group)
                        .await?
                        .into_iter()
                        .map(Message::publish)
                        .collect();
                    return Ok(messages)
                }
            }
        }
        Err(anyhow::anyhow!("User is not available to read this chat").code(403))
    }

    pub async fn unread_messages(&self, group: ObjectId, unread: i32, data: ChangeUnread) -> error::Result<()> {
        let auth = self.context.auth();
        let user_id = if let Some(org_id) = data.org_id {
            org_id.parse()?
        } else {
            auth.id().unwrap()
        };

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
