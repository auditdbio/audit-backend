use std::{fs::File, io::Write, path::Path};
use mongodb::bson::{oid::ObjectId, Bson};
use actix_files::NamedFile;
use actix_multipart::Multipart;
use futures_util::StreamExt;

use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{
        audits::PublicAudit,
        file::{ChangeFile, PublicMetadata},
    },
    context::GeneralContext,
    entities::file::{ParentEntity, ParentEntitySource, FileEntity, Metadata},
    error::{self, AddCode},
    services::{PROTOCOL, API_PREFIX, AUDITS_SERVICE},
};

const FILES_PATH_PREFIX: &str = "auditdb-files";

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
    ) -> error::Result<PublicMetadata> {
        let auth = self.context.auth();
        let current_id = auth.id();

        let mut content = vec![];

        let mut private = false;
        let mut original_name = String::new();
        let mut customer_id = String::new();
        let mut auditor_id = String::new();
        let mut full_access = String::new();

        let mut access_code: Option<String> = None;
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
                "original_name" => {
                    let mut str = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                    original_name = str;
                }
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

                "access_code" => {
                    let mut str = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                    access_code = Some(str);
                }
                "parent_entity_id" => {
                    let mut str = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                    parent_entity_id = match str.parse::<ObjectId>() {
                        Ok(id) => Some(id),
                        Err(_) => None
                    }
                }
                "parent_entity_source" => {
                    let mut str = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                    }
                    parent_entity_source = match str.parse::<ParentEntitySource>() {
                        Ok(source) => Some(source),
                        Err(_) => None
                    }
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

        if let Some(source) = parent_entity_source.clone() {
            if source == ParentEntitySource::Auditor
                || source == ParentEntitySource::Customer
                || source == ParentEntitySource::User
            {
                parent_entity_id = current_id;
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

        if file_entity == Some(FileEntity::Avatar) || file_entity == Some(FileEntity::Report) {
            is_rewritable = true;
        }

        let metas = self.context.try_get_repository::<Metadata>()?;

        if is_rewritable {
            if let Some(parent_entity) = parent_entity.clone() {
                if !auth.full_access() {
                    if file_entity == Some(FileEntity::Avatar) {
                        if parent_entity.source == ParentEntitySource::Organization {
                            // TODO: get and check user organization rights
                        } else {
                            if current_id.unwrap() != parent_entity.id {
                                return Err(anyhow::anyhow!("User is not available to add this avatar").code(403));
                            }
                        }
                    }

                    if file_entity == Some(FileEntity::Report) {
                        let audit = self
                            .context
                            .make_request::<PublicAudit>()
                            .auth(auth)
                            .get(format!(
                                "{}://{}/{}/audit/{}",
                                PROTOCOL.as_str(),
                                AUDITS_SERVICE.as_str(),
                                API_PREFIX.as_str(),
                                parent_entity.id,
                            ))
                            .send()
                            .await?
                            .json::<PublicAudit>()
                            .await?;

                        // TODO: change check for organization audits
                        if current_id.unwrap().to_hex() != audit.auditor_id {
                            return Err(anyhow::anyhow!("User is not available to add this report").code(403));
                        }
                    }
                }

                let found_metas = metas
                    .find_many("parent_entity.id", &Bson::ObjectId(parent_entity.clone().id))
                    .await?
                    .into_iter()
                    .filter(|meta| {
                        if let Some(parent) = &meta.parent_entity {
                            parent.source == parent_entity.clone().source
                        } else { false }
                    })
                    .collect::<Vec<Metadata>>();

                for meta in found_metas {
                        std::fs::remove_file(meta.path.clone()).map_or_else(
                            |e| log::info!("Failed to delete file '{}'. Error: {:?}", meta.path, e),
                            |_| log::info!("File '{}' successfully deleted.", meta.path),
                        );
                        metas.delete("id", &meta.id).await?;
                }
            } else {
                return Err(anyhow::anyhow!("Parent entity fields is required for rewritable files").code(400));
            }
        }

        let last_modified = chrono::Utc::now().timestamp_micros();
        let object_id = ObjectId::new();

        if original_name.is_empty() {
            original_name = object_id.to_hex();
        }

        let extension = if original_name.contains('.') {
            original_name.split('.').last().unwrap().to_string()
        } else {
            String::new()
        };
        original_name = original_name.rsplitn(2, '.').last().unwrap().to_string();

        let path = format!(
            "/{}/{}_{}{}",
            FILES_PATH_PREFIX,
            original_name,
            last_modified,
            if extension.is_empty() { "".to_string() } else { format!(".{}", extension) }
        );

        let os_path = Path::new(&path);

        let Some(prefix) = os_path.parent() else {
            return Err(anyhow::anyhow!("No parent directory").code(500));
        };

        std::fs::create_dir_all(prefix)?;

        let mut file = File::create(path.clone())?;
        let content = content.concat();

        let meta = Metadata {
            id: object_id,
            last_modified,
            path,
            extension,
            private,
            allowed_users: full_access,
            author: current_id,
            access_code,
            original_name: Some(original_name),
            parent_entity,
            file_entity: file_entity.clone(),
        };

        file.write_all(&content).unwrap();
        metas.insert(&meta).await?;

        Ok(meta.into())
    }


    pub async fn find_file(&self, path: String, code: Option<&String>) -> error::Result<NamedFile> {
        let auth = self.context.auth();

        let path = format!("/{}/{}", FILES_PATH_PREFIX, path);
        let metas = self.context.try_get_repository::<Metadata>()?;

        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        let is_code_match = meta.access_code.is_some() && code == meta.access_code.as_ref();
        if !Read.get_access(&auth, &meta) && !is_code_match {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        let file = match NamedFile::open_async(meta.path.clone()).await {
            Ok(file) => file,
            Err(_) => {
                NamedFile::open_async(format!("{}.{}", meta.path, meta.extension))
                    .await
                    .expect("File not found in storage")
            }
        };

        Ok(file)
    }

    pub async fn get_file_by_id(&self, file_id: ObjectId, code: Option<&String>) -> error::Result<NamedFile> {
        let auth = self.context.auth();

        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(meta) = metas.find("id", &Bson::ObjectId(file_id)).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        let is_code_match = meta.access_code.is_some() && code == meta.access_code.as_ref();
        if !Read.get_access(&auth, &meta) && !is_code_match {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        let file = NamedFile::open_async(meta.path)
            .await
            .expect("File not found in storage");

        Ok(file)
    }

    pub async fn get_meta_by_id(&self, file_id: ObjectId, code: Option<&String>) -> error::Result<PublicMetadata> {
        let auth = self.context.auth();

        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(meta) = metas.find("id", &Bson::ObjectId(file_id)).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        let is_code_match = meta.access_code.is_some() && code == meta.access_code.as_ref();
        if !Read.get_access(&auth, &meta) && !is_code_match {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        Ok(meta.into())
    }


    async fn change_file_meta(&self, mut meta: Metadata, change: ChangeFile) -> error::Result<()> {
        let auth = self.context.auth();

        if !Edit.get_access(&auth, &meta) {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        if let Some(private) = change.private {
            meta.private = private;
        }

        if change.access_code.is_some() {
            meta.access_code = change.access_code;
        }

        let metas = self.context.try_get_repository::<Metadata>()?;
        metas.delete("id", &meta.id).await?;
        metas.insert(&meta).await?;

        Ok(())
    }

    pub async fn change_file_meta_by_name(&self, path: String, change: ChangeFile) -> error::Result<()> {
        let path = format!("/{}/{}", FILES_PATH_PREFIX, path);
        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        self.change_file_meta(meta, change).await
    }

    pub async fn change_file_meta_by_id(&self, file_id: ObjectId, change: ChangeFile) -> error::Result<()> {
        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(meta) = metas.find("id", &Bson::ObjectId(file_id)).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        self.change_file_meta(meta, change).await
    }


    async fn delete_file(&self, meta: Metadata) -> error::Result<()> {
        let auth = self.context.auth();

        if !Edit.get_access(&auth, &meta) {
            return Err(anyhow::anyhow!("Access denied for this user").code(403));
        }

        std::fs::remove_file(meta.path)?;
        let metas = self.context.try_get_repository::<Metadata>()?;
        metas.delete("id", &meta.id).await?;

        Ok(())
    }

    pub async fn delete_file_by_name(&self, path: String) -> error::Result<()> {
        let path = format!("/{}/{}", FILES_PATH_PREFIX, path);
        let metas = self.context.try_get_repository::<Metadata>()?;

        let Some(meta) = metas.find("path", &Bson::String(path.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        self.delete_file(meta).await
    }

    pub async fn delete_file_by_id(&self, file_id: ObjectId) -> error::Result<()> {
        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(meta) = metas.find("id", &Bson::ObjectId(file_id)).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        self.delete_file(meta).await
    }

    pub async fn get_and_delete_by_id(&self, file_id: ObjectId) -> error::Result<NamedFile> {
        let metas = self.context.try_get_repository::<Metadata>()?;
        let Some(meta) = metas.find("id", &Bson::ObjectId(file_id)).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        let file = NamedFile::open_async(meta.path.clone())
            .await
            .unwrap();

        self.delete_file(meta.clone()).await?;

        Ok(file)
    }


    pub async fn file_by_token(&self, token: String) -> error::Result<NamedFile> {
        let tokens = self.context.try_get_repository::<FileToken>()?;

        let Some(token) = tokens.find("token", &Bson::String(token.clone())).await? else {
            return Err(anyhow::anyhow!("File not found").code(404));
        };

        let path = format!("/{}/{}", FILES_PATH_PREFIX, token.path);

        let file = NamedFile::open_async(path).await.unwrap();

        Ok(file)
    }
}
