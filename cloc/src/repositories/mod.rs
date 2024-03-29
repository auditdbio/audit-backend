use std::{collections::HashMap, path::PathBuf, sync::Arc};

use common::{
    error,
    repository::{mongo_repository::MongoRepository, Entity, Repository},
};
use mongodb::{
    bson::{doc, oid::ObjectId},
};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::handlers::cloc::ClocRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitState {
    branch: String,
    commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepoEntity {
    pub id: ObjectId,
    pub author: String,
    pub repo: String,
    pub branch: Vec<String>,
    pub commit: HashMap<String, String>,
    pub state: GitState,
}

impl Entity for GitRepoEntity {
    fn id(&self) -> ObjectId {
        todo!()
    }
}

impl GitRepoEntity {
    pub fn new(request: &ClocRequest) -> Self {
        Self {
            id: ObjectId::new(),
            author: request.author.clone(),
            repo: request.repo.clone(),
            branch: vec![request.branch.clone().unwrap_or("master".to_owned())],
            commit: HashMap::new(),
            state: GitState {
                branch: request.branch.clone().unwrap_or("master".to_owned()),
                commit: request.commit.clone(),
            },
        }
    }
}

#[derive(Clone)]
pub struct ClocRepo {
    pub git_repo: Arc<MongoRepository<GitRepoEntity>>,
    pub path: PathBuf,
}

impl ClocRepo {
    async fn update_repo(
        &self,
        original: Option<GitRepoEntity>,
        update: GitRepoEntity,
    ) -> error::Result<GitRepoEntity> {
        let Some(original) = original else {
            // fetch
            return Ok(update);
        };

        // pull
        Command::new("git")
            .arg("pull")
            .current_dir(&self.path.join(&original.author).join(&original.repo))
            .output()
            .await?;

        for branch in &update.branch {
            let fetched = original.branch.iter().find(|b| b == &branch).is_some();
            if !fetched {
                // fetch
            }
        }
        todo!()
    }

    pub async fn fetch_repo(&self, git_repo: GitRepoEntity) -> error::Result<PathBuf> {
        let found_repo = self
            .git_repo
            .collection
            .find_one(
                doc! {
                    "author": git_repo.author.clone(),
                    "repo": git_repo.repo.clone(),
                },
                None,
            )
            .await?;

        let new_repo = self.update_repo(found_repo, git_repo).await?;

        self.git_repo.insert(&new_repo).await?;

        Ok(self.path.join(&new_repo.author).join(&new_repo.repo))
    }

    pub async fn set_repository_state(&self) {
        todo!()
    }
}
