use std::{fs::File, io::Write, path::Path};

use actix_files::NamedFile;
use anyhow::bail;
use common::{
    access_rules::{AccessRules, Edit, Read},
    auth::Auth,
    context::Context,
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: ObjectId,
    pub creator_id: ObjectId,
    pub last_modified: i64,
    pub path: String,
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Metadata> for Read {
    fn get_access(object: &'a Auth, subject: &'b Metadata) -> bool {
        if let Auth::User(id) = object {
            &subject.creator_id == id
        } else {
            true
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Metadata> for Edit {
    fn get_access(object: &'a Auth, subject: &'b Metadata) -> bool {
        if let Auth::User(id) = object {
            &subject.creator_id == id
        } else {
            true
        }
    }
}

pub struct FileService {
    pub context: Context,
}

impl FileService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create_file(&self, path: String, content: Vec<u8>) -> anyhow::Result<()> {
        let Some(metas) = self.context.get_repository::<Metadata>() else {
            bail!("No metadata repository found")
        };
        let meta = metas.find("path", &Bson::String(path.clone())).await?;

        if meta.is_some() {
            bail!("File already exists")
        }

        let path = format!("/auditdb-files/{}", path);
        let path = Path::new(&path);

        let Some(prefix) = path.parent() else {
            bail!("No parent directory")
        };

        std::fs::create_dir_all(prefix)?;

        let mut file = File::create(path)?;
        file.write_all(&content).unwrap();

        let meta = Metadata {
            id: ObjectId::new(),
            creator_id: self.context.1.user_auth.as_ref().unwrap().get_id().unwrap().clone(),
            last_modified: chrono::Utc::now().timestamp(),
            path: path.to_str().unwrap().to_string(),
        };

        metas.insert(&meta).await?;

        Ok(())
    }

    pub async fn find_file(&self, path: String) -> anyhow::Result<NamedFile> {
        let Some(auth) = &self.context.1.user_auth else {
            bail!("No auth found")
        };

        let Some(metas) = self.context.get_repository::<Metadata>() else {
            bail!("No metadata repository found")
        };
        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            bail!("File not found")
        };

        if !Read::get_access(auth, &meta) {
            bail!("Access denied for this user")
        }

        let path = format!("/auditdb-files/{}", path);

        let file = actix_files::NamedFile::open_async(path).await.unwrap();

        Ok(file)
    }

    pub async fn change_file(&self, path: String, content: Vec<u8>) -> anyhow::Result<()> {
        let Some(auth) = &self.context.1.user_auth else {
            bail!("No auth found")
        };

        let Some(metas) = self.context.get_repository::<Metadata>() else {
            bail!("No metadata repository found")
        };

        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            bail!("File not found")
        };

        if !Edit::get_access(auth, &meta) {
            bail!("Access denied for this user")
        }

        let path = format!("/auditdb-files/{}", path);

        let mut file = File::create(path)?;
        file.write_all(&content)?;

        Ok(())
    }

    pub async fn delete_file(&self, path: String) -> anyhow::Result<()> {
        let Some(auth) = &self.context.1.user_auth else {
            bail!("No auth found")
        };

        let Some(metas) = self.context.get_repository::<Metadata>() else {
            bail!("No metadata repository found")
        };

        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            bail!("File not found")
        };

        if !Edit::get_access(auth, &meta) {
            bail!("Access denied for this user")
        }

        let path = format!("/auditdb-files/{}", path);

        std::fs::remove_file(path);

        Ok(())
    }
}
