use std::{collections::HashMap, sync::Arc};
use mongodb::bson::{Bson, oid::ObjectId};

use crate::repositories::file_repo::{CountResult, FileRepo, Scope};
use common::{
    api::{
        linked_accounts::LinkedService,
        user::decrypt_github_token
    },
    context::GeneralContext,
    error::{self, AddCode},
    entities::user::User,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::handlers::cloc::ClocRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClocLine {
    #[serde(alias = "nFiles")]
    files: usize,
    blank: usize,
    comment: usize,
    code: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClocCount {
    languages: HashMap<String, ClocLine>,
}

impl ClocCount {
    pub fn parse(value: Value) -> error::Result<Self> {
        let mut languages = HashMap::new();
        for (key, value) in value.as_object().unwrap() {
            if key == "header" {
                continue;
            }
            languages.insert(key.clone(), serde_json::from_value(value.clone())?);
        }
        Ok(Self { languages })
    }
}

fn process_link(link: &mut String) {
    if link.starts_with("https://github.com") || link.starts_with("http://github.com") {
        *link = link.replacen("github.com", "raw.githubusercontent.com", 1);
        *link = link.replacen("blob/", "", 1);
    }
}

pub struct ClocService {
    context: GeneralContext,
}

impl ClocService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn count(&self, request: ClocRequest) -> error::Result<CountResult> {
        let user_id = self.context.auth().id().unwrap();
        let repo = self
            .context
            .get_repository_manual::<Arc<FileRepo>>()
            .unwrap();

        let mut scope = Scope::new(request.links);
        // https://raw.githubusercontent.com/auditdbio/audit-web/942b43136ace347e69ecbd64fdda819f85775117/src/components/Chat/ImageMessage.jsx
        // https://               github.com/auditdbio/audit-web/blob/942b43136ace347e69ecbd64fdda819f85775117/src/components/Chat/ImageMessage.jsx

        for link in &mut scope.links {
            process_link(link);
        }

        // let users = self.context.try_get_repository::<User<ObjectId>>()?;
        // let Some(user) = users.find("id", &Bson::ObjectId(user_id)).await? else {
        //     return Err(anyhow::anyhow!("User not found").code(404));
        // };

        let mut access_token: Option<String> = None;
        // if let Some(linked_accounts) = user.linked_accounts {
        //     if let Some(github_account) = linked_accounts
        //         .iter()
        //         .find(|account| account.name == LinkedService::GitHub) {
        //         if let Some(encrypted_token) = github_account.token.clone() {
        //             access_token = Option::from(decrypt_github_token(encrypted_token).await?);
        //         }
        //     }
        // }

        // let (id, skiped, errors) = repo.download(user, scope.clone()).await?;
        //
        // let result = repo.count(id).await?;
        // Ok(CountResult {
        //     skiped,
        //     errors,
        //     result,
        // })

        match repo.download(user_id, scope.clone(), access_token).await {
            Ok((id, skiped, errors)) => {
                match repo.count(id).await {
                    Ok(result) => Ok(CountResult { skiped, errors, result }),
                    Err(e) => Err(
                        anyhow::anyhow!(format!("Error during count: {}", e)).code(502)
                    ),
                }
            }
            Err(e) => Err(
                anyhow::anyhow!(format!("Error during download: {}", e)).code(502)
            ),
        }
    }
}
