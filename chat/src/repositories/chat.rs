use crate::services::chat::{Message, PublicChat};
use chrono::Utc;
use common::api::chat::ChatId;

use common::repository::{Entity, Repository};
use common::{error, repository::mongo_repository::MongoRepository};
use mongodb::bson::Bson;
use mongodb::bson::{doc, oid::ObjectId, to_document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    #[serde(rename = "_id")]
    id: ObjectId,
    name: String,
    members: Vec<ChatId>,
    last_modified: i64,
    last_message: Message,
    unread: Vec<ReadId>,
}

impl Group {
    pub fn publish(self) -> PublicChat {
        PublicChat {
            id: self.id.to_hex(),
            name: self.name,
            members: self.members.into_iter().map(ChatId::publish).collect(),
            last_message: self.last_message.publish(),
            last_modified: self.last_modified,
            avatar: None,
            unread: self.unread.into_iter().map(ReadId::publish).collect(),
        }
    }
}

impl Entity for Group {
    fn id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Messages {
    #[serde(rename = "_id")]
    id: ObjectId,
    messages: Vec<Message>,
}

impl Entity for Messages {
    fn id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicReadId {
    pub id: String,
    pub unread: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadId {
    pub id: ObjectId,
    pub unread: i32,
}

impl ReadId {
    pub fn publish(self) -> PublicReadId {
        PublicReadId {
            unread: self.unread,
            id: self.id.to_hex(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateChat {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub members: [ChatId; 2],
    pub last_modified: i64,
    pub last_message: Message,
    pub unread: Option<[ReadId; 2]>,
}

impl Entity for PrivateChat {
    fn id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub enum Chat {
    Private(PrivateChat),
    Group(Group),
}

impl Chat {
    pub fn members(&self) -> Vec<ChatId> {
        match self {
            Chat::Private(private) => private.members.into_iter().collect(),
            Chat::Group(group) => group.members.clone(),
        }
    }

    pub fn chat_id(&self) -> ObjectId {
        match self {
            Chat::Private(private) => private.id,
            Chat::Group(group) => group.id,
        }
    }
}

pub struct ChatRepository {
    messages: MongoRepository<Messages>,
    groups: MongoRepository<Group>,
    private_chats: MongoRepository<PrivateChat>,
}

impl ChatRepository {
    pub fn new(
        groups: MongoRepository<Group>,
        messages: MongoRepository<Messages>,
        private_chats: MongoRepository<PrivateChat>,
    ) -> Self {
        ChatRepository {
            messages,
            groups,
            private_chats,
        }
    }

    pub async fn message(&self, message: Message) -> error::Result<Chat> {
        self.messages
            .collection
            .update_one(
                doc! {"_id": message.chat},
                doc! {"$push": {"messages": to_document(&message)?}},
                None,
            )
            .await?;

        Ok(
            if let Some(mut chat) = self.groups.delete("_id", &message.chat).await? {
                chat.last_modified = Utc::now().timestamp_micros();
                chat.last_message = message;
                self.groups.insert(&chat).await?;
                Chat::Group(chat)
            } else if let Some(mut chat) = self.private_chats.delete("_id", &message.chat).await? {
                chat.last_modified = Utc::now().timestamp_micros();
                chat.last_message = message;
                self.private_chats.insert(&chat).await?;
                Chat::Private(chat)
            } else {
                unreachable!()
            },
        )
    }

    pub async fn groups_by_user(
        &self,
        user_id: ChatId,
    ) -> error::Result<(Vec<Group>, Vec<PrivateChat>)> {
        let document = to_document(&user_id)?;
        let groups = self
            .groups
            .find_many("members", &Bson::Document(document.clone()))
            .await?;
        let chats = self
            .private_chats
            .find_many("members", &Bson::Document(document))
            .await?;
        Ok((groups, chats))
    }

    pub async fn create_private(
        &self,
        message: Message,
        other: ChatId,
    ) -> error::Result<PrivateChat> {
        let chat = PrivateChat {
            id: message.chat,
            members: [message.from, other],
            last_modified: Utc::now().timestamp_micros(),
            last_message: message.clone(),
            unread: Some([
                ReadId { id: message.from.id, unread: 0 },
                ReadId { id: other.id, unread: 0 },
            ]),
        };

        let messages = Messages {
            id: message.chat,
            messages: vec![],
        };

        self.messages.insert(&messages).await?;
        self.private_chats.insert(&chat).await?;

        Ok(chat)
    }

    pub async fn messages(&self, group: ObjectId) -> error::Result<Vec<Message>> {
        Ok(self
            .messages
            .find("_id", &Bson::ObjectId(group))
            .await?
            .map(|x| x.messages)
            .unwrap_or(vec![]))
    }

    pub async fn unread(&self, group: ObjectId, user_id: ObjectId, unread: Option<i32>) -> error::Result<()> {
        let chat = self.private_chats.find("_id", &Bson::ObjectId(group)).await?;

        if let Some(mut chat) = chat {
            if let Some(ref mut read_array) = chat.unread.as_mut() {
                if let Some(user_read) = read_array.iter_mut().find(|r| r.id == user_id) {
                    if let Some(unread) = unread {
                        user_read.unread = unread
                    } else {
                        user_read.unread = user_read.unread.clone() + 1
                    }
                }
            } else {
                chat.unread = Some([
                    ReadId { id: chat.members[0].id, unread: 0 },
                    ReadId { id: chat.members[1].id, unread: 0 },
                ])
            }

            self.private_chats.delete("_id", &group).await?;
            self.private_chats.insert(&chat).await?;
        }

        Ok(())
    }
}
