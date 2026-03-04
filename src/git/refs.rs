use std::collections::{HashMap, HashSet};

use anyhow::Result;
use git2::{BranchType, Oid, Repository};

use super::commit_for_hash;

pub struct RefInfo {
    pub name: String,
    pub ref_type: String,
    pub author: String,
    pub time: i64,
}

#[derive(Clone)]
pub struct RefLabel {
    pub name: String,
    pub kind: String,
}

pub struct HeaderRefs {
    pub branch: String,
    pub tag: Option<String>,
}

pub fn get_header_refs(repo: &Repository, ref_name: Option<&str>) -> HeaderRefs {
    if let Some(name) = ref_name {
        if let Ok(reference) = repo.find_reference(name) {
            return header_refs_for_reference(repo, &reference, name);
        }

        let branch_ref = format!("refs/heads/{}", name);
        if let Ok(reference) = repo.find_reference(&branch_ref) {
            return header_refs_for_reference(repo, &reference, name);
        }

        let tag_ref = format!("refs/tags/{}", name);
        if let Ok(reference) = repo.find_reference(&tag_ref) {
            return header_refs_for_reference(repo, &reference, name);
        }

        return HeaderRefs {
            branch: name.to_string(),
            tag: None,
        };
    }

    if let Ok(head) = repo.head() {
        if head.is_branch() {
            return HeaderRefs {
                branch: head.shorthand().unwrap_or("HEAD").to_string(),
                tag: None,
            };
        }

        if let Ok(commit) = head.peel_to_commit() {
            return HeaderRefs {
                branch: find_branch_for_commit(repo, &commit.id().to_string())
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "HEAD".to_string()),
                tag: None,
            };
        }
    }

    HeaderRefs {
        branch: "HEAD".to_string(),
        tag: None,
    }
}

pub fn refs_for_commits(
    repo: &Repository,
    commit_hashes: &[String],
) -> Result<HashMap<String, Vec<RefLabel>>> {
    let mut wanted = HashSet::new();
    let mut oid_to_hash = HashMap::new();

    for hash in commit_hashes {
        if let Ok(oid) = Oid::from_str(hash) {
            wanted.insert(oid);
            oid_to_hash.insert(oid, hash.clone());
        }
    }

    if wanted.is_empty() {
        return Ok(HashMap::new());
    }

    let mut refs: HashMap<Oid, Vec<RefLabel>> = HashMap::new();

    for_each_branch(repo, |name, target| {
        if wanted.contains(&target) {
            refs.entry(target).or_default().push(RefLabel {
                name: name.to_string(),
                kind: "branch".to_string(),
            });
        }
    })?;

    for_each_tag(repo, |name, oid| {
        if wanted.contains(&oid) {
            refs.entry(oid).or_default().push(RefLabel {
                name: name.to_string(),
                kind: "tag".to_string(),
            });
        }
    })?;

    let mut by_hash = HashMap::new();
    for (oid, labels) in refs {
        if let Some(hash) = oid_to_hash.get(&oid) {
            by_hash.insert(hash.clone(), labels);
        }
    }

    return Ok(by_hash);
}

pub fn tags_for_commit(repo: &Repository, commit_hash: &str) -> Result<Vec<String>> {
    let oid = Oid::from_str(commit_hash)?;
    let mut tags = std::collections::BTreeSet::new();

    for_each_tag(repo, |name, tag_oid| {
        if tag_oid == oid {
            tags.insert(name.to_string());
        }
    })?;

    return Ok(tags.into_iter().collect());
}

pub fn find_branch_for_commit(repo: &Repository, commit_hash: &str) -> Result<Option<String>> {
    let oid = Oid::from_str(commit_hash)?;
    let _ = commit_for_hash(repo, commit_hash)?;

    let mut result: Option<String> = None;

    for_each_branch(repo, |name, target| {
        if target == oid && result.is_none() {
            result = Some(name.to_string());
        }
    })?;

    if result.is_none() {
        for_each_branch(repo, |name, target| {
            if repo.graph_descendant_of(target, oid).unwrap_or(false) && result.is_none() {
                result = Some(name.to_string());
            }
        })?;
    }

    return Ok(result);
}

pub fn list_refs(repo: &Repository) -> Result<Vec<RefInfo>> {
    let mut branches = Vec::new();
    let mut tags = Vec::new();

    for_each_branch(repo, |name, target| {
        if let Some((author, time)) = commit_author_time(repo, target) {
            branches.push(RefInfo {
                name: name.to_string(),
                ref_type: "branch".to_string(),
                author,
                time,
            });
        }
    })?;

    for_each_tag(repo, |name, oid| {
        if let Some((author, time)) = commit_author_time(repo, oid) {
            tags.push(RefInfo {
                name: name.to_string(),
                ref_type: "tag".to_string(),
                author,
                time,
            });
        }
    })?;

    branches.sort_by(|a, b| b.time.cmp(&a.time));
    tags.sort_by(|a, b| b.time.cmp(&a.time));

    let mut refs = branches;
    refs.extend(tags);

    return Ok(refs);
}

fn branch_target(branch: &git2::Branch) -> Option<Oid> {
    return branch.get().target();
}

fn commit_author_time(repo: &Repository, oid: Oid) -> Option<(String, i64)> {
    let commit = match repo.find_commit(oid) {
        Ok(commit) => commit,
        Err(_) => return None,
    };

    let author = commit.author().name().unwrap_or("Unknown").to_string();
    let time = commit.time().seconds();

    return Some((author, time));
}

pub fn list_refs_paginated(
    repo: &Repository,
    offset: usize,
    limit: usize,
) -> Result<(Vec<RefInfo>, usize)> {
    let refs = list_refs(repo)?;
    let total = refs.len();
    if offset >= total {
        return Ok((Vec::new(), total));
    }

    let slice = refs
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();

    return Ok((slice, total));
}

fn for_each_branch(repo: &Repository, mut f: impl FnMut(&str, Oid)) -> Result<()> {
    for branch in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch?;
        let name = match branch.name()? {
            Some(name) => name,
            None => continue,
        };

        if let Some(target) = branch_target(&branch) {
            f(name, target);
        }
    }

    return Ok(());
}

fn for_each_tag(repo: &Repository, mut f: impl FnMut(&str, Oid)) -> Result<()> {
    for tag_name in repo.tag_names(None)?.iter().flatten() {
        if let Some(oid) = tag_commit_oid(repo, tag_name) {
            f(tag_name, oid);
        }
    }

    return Ok(());
}

fn tag_commit_oid(repo: &Repository, tag_name: &str) -> Option<Oid> {
    let obj = repo.revparse_single(tag_name).ok()?;
    let commit = obj.peel_to_commit().ok()?;
    return Some(commit.id());
}

fn header_refs_for_reference(
    repo: &Repository,
    reference: &git2::Reference<'_>,
    fallback_name: &str,
) -> HeaderRefs {
    if reference.is_branch() {
        HeaderRefs {
            branch: reference.shorthand().unwrap_or(fallback_name).to_string(),
            tag: None,
        }
    } else if reference.is_tag() {
        let tag = reference.shorthand().unwrap_or(fallback_name).to_string();
        let branch = reference
            .peel_to_commit()
            .ok()
            .and_then(|commit| {
                find_branch_for_commit(repo, &commit.id().to_string())
                    .ok()
                    .flatten()
            })
            .unwrap_or_else(|| "HEAD".to_string());

        HeaderRefs {
            branch,
            tag: Some(tag),
        }
    } else {
        HeaderRefs {
            branch: reference.shorthand().unwrap_or(fallback_name).to_string(),
            tag: None,
        }
    }
}
