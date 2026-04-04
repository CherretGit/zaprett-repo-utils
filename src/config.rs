use std::path::PathBuf;
use getset::Getters;
use crate::cli::CommonArgs;

#[derive(Getters)]
#[getset(get = "pub")]
pub struct Config {
    manifests_path: PathBuf,
    files_path: PathBuf,
}

impl From<CommonArgs> for Config {
    fn from(args: CommonArgs) -> Self {
        Self {
            manifests_path: args.manifests_path,
            files_path: args.files_path,
        }
    }
}