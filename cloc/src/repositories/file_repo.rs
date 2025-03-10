use std::{fs, path::PathBuf, process::Output};
use anyhow::Context;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::process::Command;

use common::{
    impl_has_last_modified,
    default_timestamp,
    auth::Auth,
    error,
    repository::{mongo_repository::MongoRepository, Entity, Repository, HasLastModified},
    services::{API_PREFIX, USERS_SERVICE, PROTOCOL},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaEntry {
    #[serde(rename = "_id")]
    id: ObjectId,
    user: ObjectId,
    links: Vec<String>,
    #[serde(default = "default_timestamp")]
    last_modified: i64,
}

impl_has_last_modified!(MetaEntry);

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
    pub skipped: Vec<String>,
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
            last_modified: Utc::now().timestamp_micros(),
        };

        // save scope and author in meta
        self.meta_repo.insert(&entry).await?;

        // make directory
        let dir_path = append_to_path(self.path.clone(), &id.to_hex());
        std::fs::create_dir_all(&dir_path).context("Failed to create directory")?;

        let path = append_to_path(self.path.clone(), &id.to_hex());
        let mut errors = vec![];
        let mut skipped = vec![];

        // download files
        for file_link in entry.links {
            let is_github_raw = file_link.starts_with("https://raw.githubusercontent.com") 
                || file_link.starts_with("http://raw.githubusercontent.com");

            let file_name = file_link.split('/').last().unwrap_or("");

            let is_html = file_name.ends_with(".html") || file_name.ends_with(".htm");

            if is_html || !is_github_raw {
                skipped.push(file_link.clone());
                continue;
            }
            
            let mut command = Command::new("wget");
            command.current_dir(&path);
            
            if is_github_raw {
                let idx = file_link.find("://").unwrap();
                let link = file_link[(idx + 3)..].to_string();
                let proxy_url = format!(
                    "{}://{}/{}/github_files/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    link,
                );
                command.arg(&proxy_url);
                command.arg("--header").arg(format!("Authorization: Bearer {}", auth.to_token()?));
            } else {
                command.arg(&file_link);
            };

            if run_command(&mut command).await.is_none() {
                errors.push(file_link.clone());
                continue;
            }

            let saved_file_path = path.join(file_name);

            if !saved_file_path.exists() {
                errors.push(file_link.clone());
                continue;
            }

            let mime_check = Command::new("file")
                .arg("--mime")
                .arg(&saved_file_path)
                .output()
                .await;
                
            if let Ok(output) = mime_check {
                let mime_output = String::from_utf8_lossy(&output.stdout);

                if mime_output.contains("html") {
                    errors.push(file_link.clone());
                    let _ = Command::new("rm")
                        .arg(&saved_file_path)
                        .output()
                        .await;
                    continue;
                }
            } else {
                errors.push(file_link.clone());
                continue;
            }
        }
        
        Ok((id, skipped, errors))
    }

    pub async fn count(&self, id: ObjectId) -> error::Result<Value> {
        let path = append_to_path(self.path.clone(), &id.to_hex());

        let entries = fs::read_dir(&path)?;
        let files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        
        if files.is_empty() {
            return Ok(serde_json::json!({}));
        }

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

        let output = match run_command(&mut command).await {
            Some(output) => output,
            None => return Err(anyhow::anyhow!("Failed to execute cloc command. Make sure cloc is installed on your system.").into()),
        };

        let output_str = String::from_utf8(output.stdout)?;
        Ok(serde_json::from_str(&output_str)?)
    }
}
