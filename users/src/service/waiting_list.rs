use common::repository::Entity;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref RUN_ACTION_SECRET: String = std::env::var("RUN_ACTION_SECRET").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingListElement {
    id: ObjectId,
    email: String,
}

impl Entity for WaitingListElement {
    fn id(&self) -> ObjectId {
        self.id
    }
}
