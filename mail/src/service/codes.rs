use common::{
    context::Context,
    entities::letter::CreateLetter,
    error,
    services::{MAIL_SERVICE, PROTOCOL},
};
use mongodb::bson::{oid::ObjectId, Bson};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Code {
    #[serde(rename = "_id")]
    id: ObjectId,
    code: String,
    user_id: ObjectId,
}

pub struct CodeService {
    pub context: Context,
}

impl CodeService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, email: String) -> error::Result<String> {
        let id = self.context.auth().id().unwrap();
        let codes = self.context.try_get_repository::<Code>()?;
        let code = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10) // Remove magic number
            .map(char::from)
            .collect::<String>();

        let code = Code {
            id: ObjectId::new(),
            code,
            user_id: *id,
        };

        let create_letter = CreateLetter {
            recipient_id: Some(*id),
            recipient_name: None,
            email,
            message: include_str!("../../templates/code.txt").replace("{code}", &code.code),
            subject: include_str!("../../templates/code_subject.txt").to_owned(),
        };
        self.context
            .make_request::<CreateLetter>()
            .auth(self.context.server_auth())
            .post(format!(
                "{}://{}/api/mail",
                PROTOCOL.as_str(),
                MAIL_SERVICE.as_str(),
            ))
            .json(&create_letter)
            .send()
            .await?;

        codes.insert(&code).await?;

        Ok(code.code)
    }

    pub async fn check(&self, code: String) -> error::Result<bool> {
        let id = self.context.auth().id().unwrap();
        let codes = self.context.try_get_repository::<Code>()?;

        let Some(code) = codes.find("code", &Bson::String(code)).await? else {
            return Ok(false);
        };

        Ok(&code.user_id == id)
    }
}
