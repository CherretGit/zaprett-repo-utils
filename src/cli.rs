use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

#[derive(Args, Clone)]
pub struct CommonArgs {
    /// Path to manifests dir
    #[arg(short, long, default_value = "manifests")]
    pub manifests_path: PathBuf,
    /// Path to files dir
    #[arg(short, long, default_value = "files")]
    pub files_path: PathBuf,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check manifests
    Check {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Update hashes
    Update {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Add new file
    New {
        /// Path to index.json
        #[arg(short, long, default_value = "index.json")]
        index_path: PathBuf,
        /// Path to new file
        file_path: PathBuf
    }
}

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}