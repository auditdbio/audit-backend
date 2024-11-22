use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Scope {
    #[serde(rename = "type")]
    pub typ: ScopeType,
    #[serde(flatten)]
    pub content: ScopeContent,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ScopeType {
    Links,
    GitBlock,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ScopeContent {
    Links(Vec<String>),
    GitBlock(GitBlock),
}

impl ScopeContent {
    pub fn links(&self) -> Vec<String> {
        match self {
            ScopeContent::Links(links) => links.clone(),
            ScopeContent::GitBlock(block) => block
                .files
                .iter()
                .map(|file| {
                    if let Some(display_url) = file.display_url.clone() {
                        display_url
                    } else {
                        format!(
                            "{}/blob/{}/{}",
                            block.repository.clone_url,
                            block.commit,
                            file.path.clone(),
                        )
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GitBlock {
    pub repository: GitBlockRepo,
    pub commit: String,
    pub files: Vec<GitBlockFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GitBlockRepo {
    pub clone_url: String,
    pub display_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GitBlockFile {
    pub path: String,
    pub display_url: Option<String>,
}
