use std::path::Path;

use anyhow::Result;
use git2::Repository;

use super::{blob_at_path, commit_for_ref};

pub struct ReadmeContent {
    pub text: String,
    pub is_markdown: bool,
}

pub struct TreeEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub last_modified: Option<i64>,
}

pub fn get_tree_entries(
    repo: &Repository,
    ref_name: Option<&str>,
    path: &str,
) -> Result<Vec<TreeEntry>> {
    let commit = commit_for_ref(repo, ref_name)?;
    let tree = commit.tree()?;

    let target_tree = if path.is_empty() {
        tree
    } else {
        let entry = tree.get_path(Path::new(path))?;
        repo.find_tree(entry.id())?
    };

    let tree_len = target_tree.len();
    let mut entries = Vec::with_capacity(tree_len);
    let last_modified = Some(commit.time().seconds());

    for entry in target_tree.iter() {
        let entry: git2::TreeEntry<'_> = entry;
        let name = match entry.name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        let is_dir = entry.kind() == Some(git2::ObjectType::Tree);
        let size = if is_dir {
            repo.find_tree(entry.id()).ok().map(|t| t.len() as u64)
        } else {
            repo.find_blob(entry.id()).ok().map(|b| b.size() as u64)
        };

        entries.push(TreeEntry {
            name,
            is_dir,
            size,
            last_modified,
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    return Ok(entries);
}

pub fn get_file_content(repo: &Repository, ref_name: Option<&str>, path: &str) -> Result<Vec<u8>> {
    let commit = commit_for_ref(repo, ref_name)?;
    let tree = commit.tree()?;
    let blob =
        blob_at_path(repo, &tree, path).ok_or_else(|| git2::Error::from_str("blob not found"))?;

    return Ok(blob.content().to_vec());
}

pub fn get_readme(repo: &Repository, ref_name: Option<&str>) -> Result<Option<ReadmeContent>> {
    let readme_names = ["README.md", "readme.md", "README", "readme"];
    let commit = commit_for_ref(repo, ref_name)?;
    let tree = commit.tree()?;

    for name in &readme_names {
        let entry: git2::TreeEntry<'_> = match tree.get_path(Path::new(name)) {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let blob = match repo.find_blob(entry.id()) {
            Ok(blob) => blob,
            Err(_) => continue,
        };

        let text = match String::from_utf8(blob.content().to_vec()) {
            Ok(text) => text,
            Err(_) => continue,
        };

        return Ok(Some(ReadmeContent {
            text,
            is_markdown: is_markdown_name(name),
        }));
    }

    return Ok(None);
}

fn is_markdown_name(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "md"))
        .unwrap_or(false)
}
