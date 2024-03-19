use std::{path::PathBuf, process::Output};

use common::{
    error,
    repository::{mongo_repository::MongoRepository, Entity, Repository},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaEntry {
    #[serde(rename = "_id")]
    id: ObjectId,
    user: ObjectId,
    links: Vec<(String, String)>,
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
    links: Vec<(String, String)>,
}

impl Scope {
    pub fn new(links: Vec<(String, String)>) -> Self {
        Self { links }
    }
}

fn append_to_path(p: PathBuf, s: &str) -> PathBuf {
    let mut p = p.into_os_string();
    p.push(s);
    p.into()
}

pub fn log_error<T>(result: Result<T, std::io::Error>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            log::error!("Command error: {}", error);
            panic!("Command error: {}", error);
        }
    }
}

pub async fn run_command(command: &mut Command) -> Output {
    log::info!("Command: {:?}", command);
    log_error(command.output().await)
}

impl FileRepo {
    pub fn new(meta_repo: MongoRepository<MetaEntry>, path: PathBuf) -> Self {
        Self { meta_repo, path }
    }

    pub async fn download(&self, user: ObjectId, files: Scope) -> error::Result<ObjectId> {
        let id = ObjectId::new();
        let entry = MetaEntry {
            id,
            user,
            links: files.links,
        };

        // save scope and author in meta
        self.meta_repo.insert(&entry).await?;
        // make directory
        run_command(
            Command::new("mkdir")
                .arg(&id.to_hex())
                .current_dir(&self.path),
        )
        .await;
        let path = append_to_path(self.path.clone(), &id.to_hex());

        // download files
        for (file_name, file_link) in entry.links {
            run_command(
                Command::new("wget")
                    .arg("-O")
                    .arg(file_name)
                    .arg(file_link)
                    .current_dir(&path),
            )
            .await;
        }
        Ok(ObjectId::new())
    }

    pub async fn count(&self, id: ObjectId, files: Scope) -> error::Result<String> {
        let path = append_to_path(self.path.clone(), &id.to_hex());

        let mut command = Command::new("cloc");
        command.arg("--json").current_dir(path);

        for (file, _) in files.links {
            command.arg(file);
        }

        let output = String::from_utf8(run_command(&mut command).await.stdout)?;
        Ok(output)
    }
}
