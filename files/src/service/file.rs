use std::{fs::File, io::Write, path::Path};

use actix_files::NamedFile;
use anyhow::bail;
use common::{
    access_rules::{AccessRules, Edit, Read},
    auth::Auth,
    context::Context,
    repository::Entity,
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: ObjectId,
    pub allowed_users: Vec<ObjectId>,
    pub last_modified: i64,
    pub path: String,
    pub extension: String,
    pub private: bool,
}

impl Entity for Metadata {
    fn id(&self) -> ObjectId {
        self.id
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Metadata> for Read {
    fn get_access(&self, auth: &'a Auth, subject: &'b Metadata) -> bool {
        match auth {
            Auth::User(id) => {
                if subject.private {
                    subject.allowed_users.contains(id)
                } else {
                    true
                }
            }
            Auth::Admin(_) | Auth::Service(_) => true,
            Auth::None => !subject.private,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Metadata> for Edit {
    fn get_access(&self, auth: &'a Auth, subject: &'b Metadata) -> bool {
        match auth {
            Auth::User(id) => {
                if subject.private {
                    subject.allowed_users.contains(id)
                } else {
                    true
                }
            }
            Auth::Admin(_) | Auth::Service(_) => true,
            Auth::None => false,
        }
    }
}

pub struct FileToken {
    pub token: String,
    pub path: String,
}

pub struct FileService {
    pub context: Context,
}

impl FileService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create_file(
        &self,
        path: String,
        allowed_users: Vec<ObjectId>,
        private: bool,
        original_name: String,
        content: Vec<u8>,
    ) -> anyhow::Result<()> {
        let Some(metas) = self.context.get_repository::<Metadata>() else {
            bail!("No metadata repository found")
        };

        let path = format!("/auditdb-files/{}", path);

        let meta = metas.find("path", &Bson::String(path.clone())).await?;

        if let Some(meta) = meta {
            metas.delete("id", &meta.id).await?;
        }

        let os_path = Path::new(&path);

        let Some(prefix) = os_path.parent() else {
            bail!("No parent directory")
        };

        std::fs::create_dir_all(prefix)?;

        let extension = original_name.split('.').last().unwrap().to_string();
        let mut file = File::create(&format!("{}.{}", path, extension))?;

        file.write_all(&content).unwrap();

        let meta = Metadata {
            id: ObjectId::new(),
            last_modified: chrono::Utc::now().timestamp(),
            path,
            extension,
            private,
            allowed_users,
        };

        metas.insert(&meta).await?;

        Ok(())
    }

    pub async fn find_file(&self, path: String) -> anyhow::Result<NamedFile> {
        let auth = self.context.auth();

        let path = format!("/auditdb-files/{}", path);

        let Some(metas) = self.context.get_repository::<Metadata>() else {
            bail!("No metadata repository found")
        };
        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            bail!("File not found")
        };

        if !Read.get_access(auth, &meta) {
            bail!("Access denied for this user")
        }
        let file = actix_files::NamedFile::open_async(format!("{}.{}", path, meta.extension))
            .await
            .unwrap();

        Ok(file)
    }

    pub async fn delete_file(&self, path: String) -> anyhow::Result<()> {
        let auth = self.context.auth();

        let Some(metas) = self.context.get_repository::<Metadata>() else {
            bail!("No metadata repository found")
        };

        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            bail!("File not found")
        };

        if !Edit.get_access(auth, &meta) {
            bail!("Access denied for this user")
        }

        let path = format!("/auditdb-files/{}", path);

        std::fs::remove_file(path)?;

        Ok(())
    }

    pub async fn create_file_token(&self) -> anyhow::Result<String> {
        todo!()
    }

    pub async fn file_by_token(&self, token: String) -> anyhow::Result<NamedFile> {
        let Some(tokens) = self.context.get_repository::<FileToken>() else {
            bail!("No token repository found")
        };

        let Some(token) = tokens.find("token", &Bson::String(token.clone())).await? else {
            bail!("File not found")
        };

        let path = format!("/auditdb-files/{}", token.path);

        let file = actix_files::NamedFile::open_async(path).await.unwrap();

        Ok(file)
    }
}
