use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::r#enum::CheckResult;
use crate::structure::{CheckRow, CheckedFile, RepoArtifact, RepoIndex, RepoIndexItem, RepoManifest};
use anyhow::Context;
use regex::Regex;
use sha256::digest;
use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};
use tabled::settings::Style;
use tabled::Table;
use walkdir::WalkDir;

pub mod cli;
mod structure;
mod r#enum;
mod config;

pub fn run(args: Cli) -> anyhow::Result<()> {
    match args.command {
        Commands::Check { manifests_path, files_path } => {
            let config = Config::new(&manifests_path, &files_path);
            check(&config)?
        },
        Commands::Update { manifests_path, files_path } => {
            let config = Config::new(&manifests_path, &files_path);
            update(&config)?
        },
        Commands::New { index_path, file_path } => { new(&index_path, &file_path)? },
    }
    Ok(())
}

fn get_index(path: &Path) -> anyhow::Result<RepoIndex> {
    let content = fs::read_to_string(path).with_context(|| {
        format!("Failed to read manifest: {}", path.display())
    })?;
    let index = serde_json::from_str(&content).with_context(|| {
        format!("Failed to parse index: {}", path.display())
    })?;
    Ok(index)
}

pub fn get_manifest(path: &Path) -> anyhow::Result<RepoManifest> {
    let content = fs::read_to_string(path).with_context(|| {
        format!("Failed to read manifest: {}", path.display())
    })?;
    let manifest = serde_json::from_str(&content).with_context( || {
        format!("Failed to parse manifest: {}", path.display())
    })?;
    Ok(manifest)
}

fn resolve_artifact_path(url: &str, files_path: &Path) -> anyhow::Result<PathBuf> {
    let re = Regex::new(r"/refs/heads/main/(files|manifests)/(.*)$")?;
    let path = re
        .captures(url)
        .and_then(|cap| cap.get(2).map(|m| m.as_str()))
        .context("Regex fail")?;
    Ok(files_path.join(path))
}

fn check_file(path: &Path, manifest_hash: &str) -> anyhow::Result<bool> {
    if !path.is_file() {
        return Ok(false)
    }
    let content = fs::read(&path)?;
    let hash = digest(&content);
    Ok(hash == *manifest_hash)
}

fn check_manifest(repo_manifest: &RepoManifest, config: &Config) -> anyhow::Result<bool> {
    let manifests_path = config.manifests_path();
    let files_path = config.files_path();
    let manifest_path = resolve_artifact_path(repo_manifest.artifact().url(), files_path)?;
    if !check_file(&manifest_path, repo_manifest.artifact().sha256())? {
        return Ok(false)
    }
    let dependency_manifest_paths = resolve_dependencies(repo_manifest, manifests_path)?;
    let dependencies: Vec<RepoManifest> = dependency_manifest_paths.iter().map(|path| {
        get_manifest(path).with_context(|| {
            format!("Failed to get dependency: {}", path.display())
        })
    }).collect::<anyhow::Result<_>>()?;
    let result = dependencies.iter().try_fold(true, |_, dep| {
        let path = resolve_artifact_path(
            dep.artifact().url(),
            files_path,
        )?;
        check_file(&path, dep.artifact().sha256())
    })?;
    Ok(result)
}

fn resolve_dependencies(repo_manifest: &RepoManifest, files_path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    repo_manifest
        .dependencies()
        .iter()
        .map(|url| resolve_artifact_path(url, files_path))
        .collect()
}

pub fn read_manifests(config: &Config) -> anyhow::Result<Vec<(RepoManifest, PathBuf)>> {
    WalkDir::new(config.manifests_path())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let file_name = e.file_name().to_string_lossy();
            !file_name.starts_with('.') && e.file_type().is_file()
        })
        .map(|entry| {
            let manifest_path = entry.path().to_path_buf();
            let manifest = get_manifest(&manifest_path).with_context(|| {
                format!("Failed to get manifest: {}", manifest_path.display())
            })?;
            Ok((manifest, manifest_path))
        })
        .collect::<anyhow::Result<_>>()
}

pub fn check(config: &Config) -> anyhow::Result<()> {
    let manifests = read_manifests(&config)?;
    let results: Vec<CheckResult> = manifests
        .iter()
        .map(|(manifest, _path) | match check_manifest(manifest, config) {
            Ok(true) => CheckResult::Ok(manifest),
            Ok(false) => CheckResult::Invalid(manifest),
            Err(e) => CheckResult::Error(manifest, e),
        })
        .collect();
    let rows: Vec<CheckRow> = results.iter().map( |result| {
        CheckRow::from(result)
    }).collect();
    let mut table = Table::new(rows);
    table.with(Style::modern());
    println!("{}", table);
    Ok(())
}

fn update(config: &Config) -> anyhow::Result<()> {
    let mut manifests = read_manifests(&config)?;
    let mut checked_files: Vec<CheckedFile> = manifests.iter_mut().map(|(manifest, manifest_path)| {
        let file_path = resolve_artifact_path(manifest.artifact().url(), config.files_path())?;
        let is_valid = check_file(&file_path, manifest.artifact().sha256())?;
        Ok(CheckedFile {
            manifest,
            manifest_path: manifest_path.to_path_buf(),
            file_path,
            is_valid,
        })
    }).collect::<anyhow::Result<_>>()?;
    for checked in checked_files.iter_mut() {
        if !checked.is_valid {
            let manifest = &mut checked.manifest;
            let content = fs::read(&checked.file_path)?;
            let hash = digest(&content);
            let mut artifact = manifest.artifact().clone();
            artifact.set_sha256(hash);
            manifest.set_artifact(artifact);
            fs::write(&checked.manifest_path, serde_json::to_string_pretty(&checked.manifest)?)?;
            println!("Updated: {}", checked.manifest_path.display())
        }
    }
    Ok(())
}

fn file_stem(path: &Path) -> anyhow::Result<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))
}

fn build_manifest_path(relative: &Path) -> anyhow::Result<PathBuf> {
    let mut path = PathBuf::from("manifests").join(relative);
    path.set_extension("json");
    Ok(path)
}

fn to_github_url(path: &Path) -> String {
    format!(
        "https://raw.githubusercontent.com/CherretGit/zaprett-repo/refs/heads/main/{}",
        path.to_string_lossy().replace('\\', "/")
    )
}

fn edit_json<T: serde::Serialize>(value: &T) -> anyhow::Result<String> {
    let json = serde_json::to_string_pretty(value)?;
    let edit = edit::edit(json)?;
    Ok(edit)
}

fn new(index_path: &PathBuf, file_path: &PathBuf) -> anyhow::Result<()> {
    if !file_path.is_file() {
        anyhow::bail!("File does not exist: {}", file_path.display())
    }
    let file_name = file_stem(&file_path)?;
    let absolute = dunce::canonicalize(file_path)?;
    let cwd = current_dir()?;
    let relative = absolute.strip_prefix(&cwd)?;
    let relative_from_files = relative.strip_prefix(Path::new("files"))?;
    let manifest_path = build_manifest_path(relative_from_files)?;
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = fs::read(&file_path)?;
    let hash = digest(&content);
    let manifest_url = to_github_url(&manifest_path);
    let file_url = to_github_url(relative);
    let index_item = RepoIndexItem::new(file_name.clone(), String::new(), manifest_url);
    let artifact = RepoArtifact::new(file_url, hash);
    let manifest = RepoManifest::new(1, file_name.clone(), String::new(), "1.0.0".to_string(), String::new(), String::new(), Vec::new(), artifact);
    let index_item_json = serde_json::from_str(&edit_json(&index_item)?)?;
    let manifest_json = edit_json(&manifest)?;
    let mut index = get_index(index_path)?;
    index.push(index_item_json);
    fs::write(&index_path, serde_json::to_string_pretty(&index)?)?;
    fs::write(&manifest_path, manifest_json)?;
    println!("Updated: index.json\nCreated: {}", manifest_path.display());
    Ok(())
}