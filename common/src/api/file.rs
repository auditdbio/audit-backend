use serde::{Deserialize, Serialize};
use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::file::{FileEntity, Metadata, ParentEntity},
    error,
    services::{API_PREFIX, FILES_SERVICE, PROTOCOL},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeFile {
    pub private: Option<bool>,
    pub access_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMetadata {
    pub id: String,
    pub last_modified: i64,
    pub path: String,
    pub extension: String,
    pub private: bool,
    pub allowed_users: Vec<String>,
    pub author: Option<String>,
    pub original_name: Option<String>,
    pub parent_entity: Option<ParentEntity<String>>,
    pub file_entity: Option<FileEntity>,
    pub is_rewritable: bool,
}

impl From<Metadata> for PublicMetadata {
    fn from(meta: Metadata) -> Self {
        let allowed_users = meta.allowed_users.iter().map(|u| u.to_hex()).collect();

        Self {
            id: meta.id.to_hex(),
            last_modified: meta.last_modified,
            path: meta.path,
            extension: meta.extension,
            private: meta.private,
            allowed_users,
            author: meta.author.map(|a| a.to_hex()),
            original_name: meta.original_name,
            parent_entity: meta.parent_entity.map(|e| e.stringify()),
            file_entity: meta.file_entity,
            is_rewritable: meta.is_rewritable,
        }
    }
}

pub async fn request_file_metadata(
    context: &GeneralContext,
    id: String,
    auth: Auth,
) -> error::Result<Option<PublicMetadata>> {
    let res = context
        .make_request::<PublicMetadata>()
        .get(format!(
            "{}://{}/{}/file/meta/{}",
            PROTOCOL.as_str(),
            FILES_SERVICE.as_str(),
            API_PREFIX.as_str(),
            id,
        ))
        .auth(auth)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(Some(res.json::<PublicMetadata>().await?))
    } else {
        Ok(None)
    }
}
