use anyhow::bail;
use common::{
    context::Context,
    entities::letter::CreateLetter,
    services::{MAIL_SERVICE, PROTOCOL}, repository::Entity,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static!{
    static ref RUN_ACTION_SECRET: String = std::env::var("RUN_ACTION_SECRET").unwrap();
}

pub struct WaitingListService {
    context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingListElement {
    id: ObjectId,
    email: String,
}

impl Entity for WaitingListElement {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

impl WaitingListService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn run_action(&self, secret: String) -> anyhow::Result<()> {
        if secret != *RUN_ACTION_SECRET {
            return Ok(());
        }

        let Some(waiting_list) = self.context.get_repository::<WaitingListElement>() else {
            bail!("No waiting list repository found")
        };

        for element in waiting_list.find_all(0, 100).await? {
            let letter = CreateLetter {
                email: element.email,
                subject: "Welcome to our service!".to_string(),
                message: include_str!("../../templates/welcome.txt").to_string(),
            };

            self.context
                .make_request()
                .auth(self.context.server_auth())
                .post(format!(
                    "{}://{}/api/mail",
                    PROTOCOL.as_str(),
                    MAIL_SERVICE.as_str()
                ))
                .json(&letter)
                .send()
                .await?;
        }

        Ok(())
    }
}
