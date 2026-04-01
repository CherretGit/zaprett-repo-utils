use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Check manifests
    Check {
        /// Path to manifests dir
        #[arg(short, long, default_value = "manifests")]
        manifests_path: PathBuf,
        /// Path to files dir
        #[arg(short, long, default_value = "files")]
        files_path: PathBuf,
    },
    /// Update hashes
    Update {
        /// Path to manifests dir
        #[arg(short, long, default_value = "manifests")]
        manifests_path: PathBuf,
        /// Path to files dir
        #[arg(short, long, default_value = "files")]
        files_path: PathBuf,
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