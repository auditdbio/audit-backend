use common::{context::Context, error, repository::Entity};
use mongodb::bson::{oid::ObjectId, Bson};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Code {
    #[serde(rename = "_id")]
    id: ObjectId,
    code: String,
    payload: String,
}

impl Entity for Code {
    fn id(&self) -> ObjectId {
        self.id
    }
}

pub struct CodeService {
    pub context: Context,
}

impl CodeService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, payload: String) -> error::Result<String> {
        let codes = self.context.try_get_repository::<Code>()?;
        let code = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20) // TODO: Remove magic number
            .map(char::from)
            .collect::<String>();

        let code = Code {
            id: ObjectId::new(),
            code,
            payload,
        };

        codes.insert(&code).await?;

        Ok(code.code)
    }

    pub async fn check(&self, code: String) -> error::Result<Option<String>> {
        let codes = self.context.try_get_repository::<Code>()?;

        let Some(code) = codes.find("code", &Bson::String(code)).await? else {
            return Ok(None);
        };

        Ok(Some(code.payload))
    }
}
