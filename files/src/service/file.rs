use std::{fs::File, io::Write, path::Path};
use actix_files::NamedFile;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

use common::{
    impl_has_last_modified,
    access_rules::{AccessRules, Edit, Read},
    auth::Auth,
    context::GeneralContext,
    entities::file::{ParentEntity, FileEntity},
    error::{self, AddCode},
    repository::{Entity, HasLastModified},
};
use common::entities::file::ParentEntitySource;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    pub id: ObjectId,
    pub allowed_users: Vec<ObjectId>,
    pub last_modified: i64,
    pub path: String,
    pub extension: String,
    pub private: bool,
    pub original_name: Option<String>,
    pub parent_entity: Option<ParentEntity>,
    pub file_entity: Option<FileEntity>,
    #[serde(default)]
    pub is_rewritable: bool,
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
            Auth::User(id) => {
                if subject.private {
                    subject.allowed_users.contains(id)
                } else {
                    true
                }
            }
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
        mut payload: Multipart,
    ) -> error::Result<()> {
        let mut content = vec![];

        let mut private = false;
        let mut original_name = String::new();
        let mut customer_id = String::new();
        let mut auditor_id = String::new();
        let mut full_access = String::new();

        let mut parent_entity_id: Option<ObjectId> = None;
        let mut parent_entity_source: Option<ParentEntitySource> = None;
        let mut file_entity: Option<FileEntity> = None;
        let mut is_rewritable = false;

        while let Some(item) = payload.next().await {
            let mut field = item.unwrap();

            match field.name() {
                "file" => {
                    let content_disposition = field.content_disposition();
                    if let Some(name) = content_disposition.get_filename() {
                        original_name = name.to_string();
                    }
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        content.push(data);
                    }
                }
                // "path" => {
                //     while let Some(chunk) = field.next().await {
                //         let data = chunk.unwrap();
                //         path.push_str(&String::from_utf8(data.to_vec()).unwrap());
                //     }
                // }
                // "original_name" => {
                //     while let Some(chunk) = field.next().await {
                //         let data = chunk.unwrap();
                //         original_name.push_str(&String::from_utf8(data.to_vec()).unwrap());
                //     }
                // }
                "private" => {
                    let mut str = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                    private = str == "true";
                }
                "customerId" => {
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        customer_id.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                }
                "auditorId" => {
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        auditor_id.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                }
                "full_access" => {
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        full_access.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                }

                "parent_entity_id" => {
                    let mut str = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                    parent_entity_id = Some(str.parse()?)
                }
                "parent_entity_source" => {
                    let mut str = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                    parent_entity_source = Some(str.parse().unwrap());
                }
                "file_entity" => {
                    let mut str = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                    file_entity = Some(str.parse().unwrap());
                }
                _ => (),
            }
        }

        let mut full_access = full_access
            .split(' ')
            .filter_map(|id| id.trim().parse().ok())
            .collect::<Vec<ObjectId>>();

        if private {
            if let Ok(customer_id) = customer_id.parse() {
                full_access.push(customer_id);
            }

            if let Ok(auditor_id) = auditor_id.parse() {
                full_access.push(auditor_id);
            }
        }

        let parent_entity = if parent_entity_id.is_some() && parent_entity_source.is_some() {
            Some(ParentEntity {
                id: parent_entity_id.unwrap(),
                source: parent_entity_source.unwrap(),
            })
        } else {
            None
        };

        let metas = self.context.try_get_repository::<Metadata>()?;

        let last_modified = chrono::Utc::now().timestamp_micros();
        let object_id = ObjectId::new();
        if original_name.is_empty() {
            original_name = object_id.to_hex();
        }
        let path = format!("/auditdb-files/{}{}", original_name, last_modified);

        if file_entity == Some(FileEntity::Avatar) || file_entity == Some(FileEntity::Report) {
            is_rewritable = true;
        }

        if is_rewritable && parent_entity.is_some() {
            let found_metas = metas
                .find_many("parent_entity.id", &Bson::ObjectId(parent_entity_id.unwrap()))
                .await?;

            if let Some(meta) = found_metas
                .iter()
                .find(|m: &&Metadata| {
                    if let Some(parent) = &m.parent_entity {
                        parent.source == parent_entity.clone().unwrap().source
                    } else { false }
                })
            {
                std::fs::remove_file(meta.path.clone())?;
                metas.delete("id", &meta.id).await?;
            }
        }

        let os_path = Path::new(&path);

        let Some(prefix) = os_path.parent() else {
            return Err(anyhow::anyhow!("No parent directory").code(500));
        };

        std::fs::create_dir_all(prefix)?;

        let extension = if original_name.contains('.') {
            original_name.split('.').last().unwrap().to_string()
        } else {
            String::new()
        };
        original_name = original_name.rsplitn(2, '.').last().unwrap().to_string();
        let mut file = File::create(format!("{}.{}", path, extension))?;

        let content = content.concat();
        file.write_all(&content).unwrap();

        let meta = Metadata {
            id: object_id,
            last_modified,
            path,
            extension,
            private,
            allowed_users: full_access,
            original_name: Some(original_name),
            parent_entity,
            file_entity,
            is_rewritable,
        };

        metas.insert(&meta).await?;

        Ok(())
    }

    pub async fn find_file(&self, path: String) -> error::Result<NamedFile> {
        let auth = self.context.auth();

        let path = format!("/auditdb-files/{}", path);

        let metas = self.context.try_get_repository::<Metadata>()?;

        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        if !Read.get_access(&auth, &meta) {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        let file = NamedFile::open_async(format!("{}.{}", path, meta.extension))
            .await
            .unwrap();

        Ok(file)
    }

    pub async fn get_file_by_id(&self, file_id: ObjectId) -> error::Result<NamedFile> {
        let auth = self.context.auth();

        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(meta) = metas.find("id", &Bson::ObjectId(file_id)).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        if !Read.get_access(&auth, &meta) {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        let file = NamedFile::open_async(format!("{}.{}", meta.path, meta.extension))
            .await
            .unwrap();

        Ok(file)
    }

    pub async fn get_meta_by_id(&self, file_id: ObjectId) -> error::Result<Metadata> {
        let auth = self.context.auth();

        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(meta) = metas.find("id", &Bson::ObjectId(file_id)).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        if !Read.get_access(&auth, &meta) {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        Ok(meta)
    }

    pub async fn delete_file(&self, path: String) -> error::Result<()> {
        let auth = self.context.auth();

        let metas = self.context.try_get_repository::<Metadata>()?;

        let path = format!("/auditdb-files/{}", path);

        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        if !Edit.get_access(&auth, &meta) {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        std::fs::remove_file(path)?;

        Ok(())
    }

    pub async fn delete_file_by_id(&self, file_id: ObjectId) -> error::Result<()> {
        let auth = self.context.auth();

        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(meta) = metas.find("id", &Bson::ObjectId(file_id)).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        if !Edit.get_access(&auth, &meta) {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        std::fs::remove_file(meta.path)?;

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
