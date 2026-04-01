use crate::structure::RepoManifest;

pub enum CheckResult<'a> {
    Ok(&'a RepoManifest),
    Invalid(&'a RepoManifest),
    Error(&'a RepoManifest, anyhow::Error),
}