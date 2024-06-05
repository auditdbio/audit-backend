use std::{collections::HashMap, sync::Arc};

use crate::repositories::file_repo::{CountResult, FileRepo, Scope};
use common::{
    context::GeneralContext,
    error::{self, AddCode},
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
        *link = link
            .replacen("github.com", "raw.githubusercontent.com", 1)
            .replacen("blob/", "", 1);

        if let Some(index) = link.find("://") {
            *link = link[(index + 3)..].to_string();
        }
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
        let auth = self.context.auth();
        let user_id = auth.id().unwrap();

        let repo = self
            .context
            .get_repository_manual::<Arc<FileRepo>>()
            .unwrap();

        let mut scope = Scope::new(request.links);

        for link in &mut scope.links {
            process_link(link);
        }

        match repo.download(user_id, scope.clone(), auth).await {
            Ok((id, skiped, errors)) => {
                match repo.count(id).await {
                    Ok(result) => Ok(CountResult { skiped, errors, result }),
                    Err(e) => Err(
                        anyhow::anyhow!(format!("Error during count: {}", e)).code(502)
                    ),
                }
            }
            Err(e) => Err(anyhow::anyhow!(format!("Error during download: {}", e)).code(502)),
        }
    }
}
