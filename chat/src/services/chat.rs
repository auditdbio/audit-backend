use common::{
    api::chat::PublicMessage,
    context::Context,
    entities::{auditor::PublicAuditor, customer::PublicCustomer},
    error,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

pub struct Chat {
    context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewMan {
    Auditor(PublicAuditor),
    Customer(PublicCustomer),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preview {
    content: Vec<(PreviewMan, String)>,
}

impl Chat {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn send_message(&self, message: PublicMessage) -> error::Result<()> {
        // save
        // send event
        todo!()
    }

    pub async fn preview(&self) -> error::Result<Preview> {
        todo!()
    }

    pub async fn messages(&self, sender: ObjectId) -> error::Result<Vec<String>> {
        todo!()
    }
}
