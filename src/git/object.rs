use std::path::Path;

use anyhow::Result;
use git2::{Blob, Commit, Oid, Repository, Tree};

pub fn commit_for_ref<'repo>(
    repo: &'repo Repository,
    ref_name: Option<&str>,
) -> Result<Commit<'repo>> {
    repo.revparse_single(ref_name.unwrap_or("HEAD"))?
        .peel_to_commit()
        .map_err(Into::into)
}

pub fn commit_for_hash<'repo>(repo: &'repo Repository, hash: &str) -> Result<Commit<'repo>> {
    repo.find_commit(Oid::from_str(hash)?).map_err(Into::into)
}

pub fn parent_tree<'repo>(commit: &Commit<'repo>) -> Option<Tree<'repo>> {
    commit.parent(0).ok()?.tree().ok()
}

pub fn blob_at_path<'repo>(
    repo: &'repo Repository,
    tree: &Tree<'_>,
    path: &str,
) -> Option<Blob<'repo>> {
    tree.get_path(Path::new(path))
        .ok()
        .and_then(|entry| repo.find_blob(entry.id()).ok())
}
