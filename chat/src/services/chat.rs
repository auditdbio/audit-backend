use common::{
    api::{
        chat::PublicMessage,
        events::{EventPayload, PublicEvent},
    },
    context::Context,
    entities::{auditor::PublicAuditor, customer::PublicCustomer},
    error,
    services::{EVENTS_SERVICE, PROTOCOL},
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

pub struct ChatService {
    context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewMan {
    Auditor(PublicAuditor),
    Customer(PublicCustomer),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    #[serde(rename = "_id")]
    id: ObjectId,
    name: String,
    members: Vec<ObjectId>,
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicGroup {
    id: String,
    name: String,
    members: Vec<String>,
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preview {
    content: Vec<(PublicGroup, Message)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    id: ObjectId,
    from: ObjectId,
    group: ObjectId,
    time: i64,
}

impl ChatService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn send_message(&self, public_message: PublicMessage) -> error::Result<()> {
        // TODO: check permissions
        let groups = self.context.try_get_repository::<Group>()?;

        let group_id = public_message.group.parse()?;

        let mut group = groups.delete("_id", &group_id).await?.unwrap();

        let message = Message {
            id: public_message.id.parse()?,
            from: public_message.from.parse()?,
            group: group_id,
            time: public_message.time,
        };

        group.messages.push(message);

        let payload = EventPayload::ChatMessage(public_message);

        for user_id in group.members {
            let event = PublicEvent::new(user_id, payload.clone());

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

    pub async fn preview(&self) -> error::Result<Preview> {
        let auth = self.context.auth();

        let id = auth.id().unwrap();

        let groups = self.context.try_get_repository::<Group>()?;

        todo!()
    }

    pub async fn messages(&self, group: ObjectId) -> error::Result<Vec<String>> {
        todo!()
    }
}
