use std::collections::HashMap;

use crate::repositories::{ClocRepo, GitRepoEntity};
use common::{context::GeneralContext, error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::process::Command;

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

    pub async fn count(&self, request: ClocRequest) -> error::Result<ClocCount> {
        let git_repo = GitRepoEntity::new(&request);
        let repo = self.context.get_repository_manual::<ClocRepo>().unwrap();
        let path = repo.fetch_repo(git_repo).await?;
        let result = Command::new("cloc")
            .arg(path)
            .arg("--json")
            .output()
            .await?;
        let res: Value = serde_json::from_slice(&result.stdout)?;
        Ok(ClocCount::parse(res)?)
    }
}
