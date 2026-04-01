use std::path::PathBuf;
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use crate::r#enum::CheckResult;

#[derive(Serialize, Deserialize, Setters, Getters)]
#[getset(set = "pub", get = "pub")]
pub struct RepoIndex {
    schema: i32,
    items: Vec<RepoIndexItem>
}

#[derive(Clone, Serialize, Deserialize, Setters, Getters)]
#[getset(set = "pub", get = "pub")]
pub struct RepoIndexItem {
    id: String,
    r#type: String,
    manifest: String,
}

#[derive(Serialize, Deserialize, Setters, Getters)]
#[getset(set = "pub", get = "pub")]
pub struct RepoManifest{
    schema: i32,
    id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    dependencies: Vec<String>,
    artifact: RepoArtifact
}

#[derive(Serialize, Deserialize, Setters, Getters)]
#[getset(set = "pub", get = "pub")]
#[derive(Clone)]
pub struct RepoArtifact {
    url: String,
    sha256: String
}

pub struct CheckedFile<'a> {
    pub manifest: &'a mut RepoManifest,
    pub manifest_path: PathBuf,
    pub file_path: PathBuf,
    pub is_valid: bool,
}

#[derive(Getters, Tabled)]
#[getset(get = "pub")]
pub struct CheckRow {
    status: String,
    id: String,
    details: String
}

impl RepoIndex {
    pub fn push(&mut self, item: RepoIndexItem) {
        self.items.push(item);
    }
}

impl RepoIndexItem {
    pub fn new(id: String, r#type: String, manifest: String) -> Self {
        Self {
            id,
            r#type,
            manifest
        }
    }
}

impl RepoManifest {
    pub fn new(schema: i32, id: String, name: String, version: String, author: String, description: String, dependencies: Vec<String>, artifact: RepoArtifact) -> Self {
        Self {
            schema,
            id,
            name,
            version,
            author,
            description,
            dependencies,
            artifact
        }
    }
}

impl RepoArtifact {
    pub fn new(url: String, sha256: String) -> Self {
        Self {
            url,
            sha256
        }
    }
}

impl<'a> From<&CheckResult<'a>> for CheckRow {
    fn from(result: &CheckResult<'a>) -> Self {
        match result {
            CheckResult::Ok(m) => Self {
                status: "Ok".to_string(),
                id: m.id.clone(),
                details: "-".to_string()
            },
            CheckResult::Invalid(m) => Self {
                status: "Invalid".to_string(),
                id: m.id.clone(),
                details: "Manifest check fail".to_string()
            },
            CheckResult::Error(m, e) => Self {
                status: "Error".to_string(),
                id: m.id.clone(),
                details: format!("{}", e)
            }
        }
    }
}