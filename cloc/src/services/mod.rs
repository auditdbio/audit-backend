use std::{collections::HashMap, sync::Arc};

use crate::repositories::file_repo::{FileRepo, Scope};
use common::{context::GeneralContext, error};
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

pub struct ClocService {
    context: GeneralContext,
}

impl ClocService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn count(&self, request: ClocRequest) -> error::Result<String> {
        let user = self.context.auth().id().unwrap();
        let repo = self
            .context
            .get_repository_manual::<Arc<FileRepo>>()
            .unwrap();
        let scope = Scope::new(
            request
                .links
                .into_iter()
                .enumerate()
                .map(|(i, link)| (format!("file_name{}", i), link))
                .collect(),
        );
        let id = repo.download(user, scope.clone()).await?;

        repo.count(user, scope).await
    }
}
