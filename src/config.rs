use std::path::Path;
use getset::Getters;

#[derive(Getters)]
#[getset(get = "pub")]
pub struct Config<'a> {
    manifests_path: &'a Path,
    files_path: &'a Path,
}

impl<'a> Config<'a> {
    pub fn new(manifests_path: &'a Path, files_path: &'a Path) -> Self {
        Self { manifests_path, files_path }
    }
}