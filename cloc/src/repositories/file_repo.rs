use std::{fs, path::PathBuf, process::Output};
use anyhow::Context;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::process::Command;

use common::{
    auth::Auth,
    error,
    repository::{mongo_repository::MongoRepository, Entity, Repository},
    services::{API_PREFIX, USERS_SERVICE, PROTOCOL},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaEntry {
    #[serde(rename = "_id")]
    id: ObjectId,
    user: ObjectId,
    links: Vec<String>,
}

impl Entity for MetaEntry {
    fn id(&self) -> ObjectId {
        self.id
    }
}

pub struct FileRepo {
    pub meta_repo: MongoRepository<MetaEntry>,
    path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub links: Vec<String>,
}

impl Scope {
    pub fn new(links: Vec<String>) -> Self {
        Self { links }
    }
}

fn append_to_path(p: PathBuf, s: &str) -> PathBuf {
    let mut p = p.into_os_string();
    p.push(format!("/{s}"));
    p.into()
}

pub fn log_error<T>(result: Result<T, std::io::Error>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            log::error!("Command error: {}", error);
            None
        }
    }
}

pub async fn run_command(command: &mut Command) -> Option<Output> {
    log::error!("Command: {:?}", command);
    log_error(command.output().await)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountResult {
    pub skiped: Vec<String>,
    pub errors: Vec<String>,
    pub result: Value,
}

impl FileRepo {
    pub fn new(meta_repo: MongoRepository<MetaEntry>, path: PathBuf) -> Self {
        Self { meta_repo, path }
    }

    pub async fn download(
        &self,
        user_id: ObjectId,
        files: Scope,
        auth: Auth,
    ) -> error::Result<(ObjectId, Vec<String>, Vec<String>)> {
        let id = ObjectId::new();
        let entry = MetaEntry {
            id,
            user: user_id,
            links: files.links,
        };

        // save scope and author in meta
        self.meta_repo.insert(&entry).await?;
        // make directory
        run_command(
            Command::new("mkdir")
                .arg(&id.to_hex())
                .current_dir(&self.path),
        ).await.context("Failed to create directory")?;

        let path = append_to_path(self.path.clone(), &id.to_hex());
        let mut errors = vec![];
        let mut skiped = vec![];
        // download files
        for file_link in entry.links {
            // if run_command(Command::new("wget").arg(&file_link).current_dir(&path))
            //     .await
            //     .is_none()
            // {
            //     errors.push(file_link);
            //     continue;
            // }

            let mut command = Command::new("wget");
            command.current_dir(&path);
            if file_link.starts_with("raw.githubusercontent.com") {
                let proxy_url = format!(
                    "{}://{}/{}/github_files/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    file_link
                );
                command.arg(&proxy_url);
                command.arg("--header").arg(format!("Authorization: Bearer {}", auth.to_token()?));
            } else {
                command.arg(&file_link);
            };

            if run_command(&mut command)
                .await
                .is_none()
            {
                errors.push(file_link.clone());
            }

            let file_name = file_link.split('/').last().unwrap();
            let saved_file_path = path.join(file_name);

            // let basename = String::from_utf8(
            //     Command::new("file")
            //         .arg("--mime")
            //         .arg(saved_file_path)
            //         .output()
            //         .await
            //         .context("Failed to get file mime type")?
            //         .stdout,
            // ).context("Failed to parse basename")?;
            //
            // let html_check_output = String::from_utf8(
            //     Command::new("file")
            //         .arg("--mime")
            //         .arg(basename)
            //         .current_dir(path.clone())
            //         .output()
            //         .await
            //         .context("Failed to check file mime type")?
            //         .stdout,
            // ).context("Failed to parse HTML check output")?;
            //
            // if html_check_output.contains("html") {
            //     skiped.push(file_link);
            //     continue;
            // }
        }
        Ok((id, skiped, errors))
    }

    pub async fn count(&self, id: ObjectId) -> error::Result<Value> {
        let path = append_to_path(self.path.clone(), &id.to_hex());

        let mut command = Command::new("cloc");
        command.arg("--json").current_dir(path.clone());

        // get all files in directory
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if !file_path.is_dir() {
                command.arg(file_path.file_name().unwrap());
            }
        }

        let output = String::from_utf8(run_command(&mut command).await.unwrap().stdout)?;
        Ok(serde_json::from_str(&output)?)
    }
}
