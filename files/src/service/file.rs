use std::{fs::File, io::Write, path::Path};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};
use actix_files::NamedFile;

use common::{
    impl_has_last_modified,
    access_rules::{AccessRules, Edit, Read},
    api::file::ChangeFile,
    auth::Auth,
    context::GeneralContext,
    error::{self, AddCode},
    repository::{Entity, HasLastModified},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: ObjectId,
    pub allowed_users: Vec<ObjectId>,
    pub last_modified: i64,
    pub path: String,
    pub extension: String,
    pub private: bool,
    pub access_code: Option<String>,
}

impl_has_last_modified!(Metadata);

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
            Auth::Admin(_) | Auth::Service(_, _) => true,
            Auth::None => !subject.private,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Metadata> for Edit {
    fn get_access(&self, auth: &'a Auth, subject: &'b Metadata) -> bool {
        match auth {
            Auth::User(id) => subject.allowed_users.contains(id),
            Auth::Admin(_) | Auth::Service(_, _) => true,
            Auth::None => false,
        }
    }
}

pub struct FileToken {
    pub token: String,
    pub path: String,
}

pub struct FileService {
    pub context: GeneralContext,
}

impl FileService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn create_file(
        &self,
        path: String,
        allowed_users: Vec<ObjectId>,
        private: bool,
        original_name: String,
        content: Vec<u8>,
        access_code: String,
    ) -> error::Result<()> {
        let metas = self.context.try_get_repository::<Metadata>()?;

        let path = format!("/auditdb-files/{}", path);
        let meta = metas.find("path", &Bson::String(path.clone())).await?;

        if let Some(meta) = meta {
            metas.delete("id", &meta.id).await?;
        }

        let os_path = Path::new(&path);

        let Some(prefix) = os_path.parent() else {
            return Err(anyhow::anyhow!("No parent directory").code(500));
        };

        std::fs::create_dir_all(prefix)?;

        let extension = original_name.split('.').last().unwrap().to_string();
        let mut file = File::create(format!("{}.{}", path, extension))?;

        file.write_all(&content).unwrap();

        let mut meta = Metadata {
            id: ObjectId::new(),
            last_modified: chrono::Utc::now().timestamp_micros(),
            path,
            extension,
            private,
            allowed_users,
            access_code: None,
        };

        if !access_code.is_empty() {
            meta.access_code = Some(access_code);
        }

        metas.insert(&meta).await?;

        Ok(())
    }

    pub async fn find_file(&self, path: String, code: Option<&String>) -> error::Result<NamedFile> {
        let auth = self.context.auth();

        let path = format!("/auditdb-files/{}", path);
        let metas = self.context.try_get_repository::<Metadata>()?;

        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        let is_code_match = code == meta.access_code.as_ref();

        if !Read.get_access(&auth, &meta) && !is_code_match {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }
        let file = NamedFile::open_async(format!("{}.{}", path, meta.extension))
            .await
            .unwrap();

        Ok(file)
    }

    pub async fn delete_file(&self, path: String) -> error::Result<()> {
        let auth = self.context.auth();

        let path = format!("/auditdb-files/{}", path);
        let metas = self.context.try_get_repository::<Metadata>()?;

        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        if !Edit.get_access(&auth, &meta) {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        let path = format!("/auditdb-files/{}", path);

        std::fs::remove_file(path)?;
        metas.delete("id", &meta.id).await?;

        Ok(())
    }

    pub async fn change_file(&self, path: String, change: ChangeFile) -> error::Result<()> {
        let auth = self.context.auth();

        let path = format!("/auditdb-files/{}", path);
        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(mut meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        if !Edit.get_access(&auth, &meta) {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        if let Some(private) = change.private {
            meta.private = private;
        }

        if change.access_code.is_some() {
            meta.access_code = change.access_code;
        }

        metas.delete("id", &meta.id).await?;
        metas.insert(&meta).await?;

        Ok(())
    }

    pub async fn file_by_token(&self, token: String) -> error::Result<NamedFile> {
        let tokens = self.context.try_get_repository::<FileToken>()?;

        let Some(token) = tokens.find("token", &Bson::String(token.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        let path = format!("/auditdb-files/{}", token.path);

        let file = NamedFile::open_async(path).await.unwrap();

        Ok(file)
    }
}
