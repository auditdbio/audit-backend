use common::{default_timestamp, impl_has_last_modified, repository::{Entity, HasLastModified}};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref RUN_ACTION_SECRET: String = std::env::var("RUN_ACTION_SECRET").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingListElement {
    id: ObjectId,
    email: String,
    #[serde(default = "default_timestamp")]
    pub last_modified: i64,
}

impl_has_last_modified!(WaitingListElement);

impl Entity for WaitingListElement {
    fn id(&self) -> ObjectId {
        self.id
    }
}
