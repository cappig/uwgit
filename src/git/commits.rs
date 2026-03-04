use anyhow::Result;
use git2::{Commit, Repository};

use super::{commit_for_hash, commit_for_ref};

pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub author_email: String,
    pub message: String,
    pub time: i64,
}

pub fn get_commits_paginated(
    repo: &Repository,
    ref_name: Option<&str>,
    offset: usize,
    limit: usize,
) -> Result<(Vec<CommitInfo>, usize)> {
    let commit = commit_for_ref(repo, ref_name)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push(commit.id())?;

    let mut commits = Vec::with_capacity(limit);
    let mut total = 0usize;

    for (idx, oid) in revwalk.enumerate() {
        let oid = oid?;
        total += 1;
        if idx >= offset && commits.len() < limit {
            let commit = repo.find_commit(oid)?;
            commits.push(commit_to_info(&commit));
        }
    }

    return Ok((commits, total));
}

pub fn get_commit(repo: &Repository, hash: &str) -> Result<CommitInfo> {
    let commit = commit_for_hash(repo, hash)?;
    return Ok(commit_to_info(&commit));
}

fn commit_to_info(commit: &Commit) -> CommitInfo {
    let author = commit.author();
    let hash = commit.id().to_string();

    return CommitInfo {
        hash: hash.clone(),
        short_hash: hash.get(..8).unwrap_or(&hash).to_string(),
        author: author.name().unwrap_or("Unknown").to_string(),
        author_email: author.email().unwrap_or("").to_string(),
        message: commit.message().unwrap_or("").to_string(),
        time: commit.time().seconds(),
    };
}
