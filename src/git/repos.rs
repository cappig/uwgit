use std::path::Path;

use anyhow::{Context, Result};
use git2::Repository;

pub struct RepoInfo {
    pub name: String,
    pub description: Option<String>,
}

pub fn list_repos(base_path: &Path) -> Result<Vec<RepoInfo>> {
    let base = std::fs::canonicalize(base_path)?;
    let mut repos = Vec::new();

    let entries: Vec<_> = std::fs::read_dir(base_path)?.collect();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let canonical = match std::fs::canonicalize(&path) {
            Ok(canonical) => canonical,
            Err(_) => continue,
        };

        if !canonical.starts_with(&base) {
            continue;
        }

        let name = entry.file_name().to_str().unwrap_or("").to_string();
        let description = std::fs::read_to_string(canonical.join("description")).ok();

        repos.push(RepoInfo { name, description });
    }

    repos.sort_by(|a, b| a.name.cmp(&b.name));

    return Ok(repos);
}

pub fn open_repo(path: &Path) -> Result<Repository> {
    return Repository::open(path).context("Failed to open repository");
}
