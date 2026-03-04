use std::fmt::Write as _;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Html;

use crate::git;
use crate::templates::{CommitDiffBodyTemplate, CommitFileSummary, CommitTemplate};

use super::util::{RepoRequestContext, display_time, is_safe_repo_path, render_template};
use super::{AppError, AppState, CommitDiffQuery, CommitQuery};

pub async fn commit(
    Path((repo_name, commit_hash)): Path<(String, String)>,
    Query(query): Query<CommitQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let ctx = RepoRequestContext::load(&state, repo_name, query.ref_name.clone())?;
    let chrome = ctx.chrome.clone();
    let nav = ctx.nav("commit");

    render_template(move || {
        let commit_info =
            git::get_commit(&ctx.repo, &commit_hash).map_err(|_| AppError::NotFound)?;
        let commit_message = commit_info.message.trim_end().to_string();
        let (files_raw, diff_stats) = git::get_commit_diff_summaries(&ctx.repo, &commit_hash)
            .map_err(|_| AppError::NotFound)?;

        Ok(CommitTemplate {
            chrome,
            nav,
            commit_title: commit_message.lines().next().unwrap_or("").to_string(),
            commit_message,
            commit_hash: commit_info.hash,
            branch: resolve_branch(&ctx.repo, &commit_hash, ctx.git_ref()),
            tags: git::tags_for_commit(&ctx.repo, &commit_hash).map_err(|_| AppError::NotFound)?,
            author: commit_info.author,
            author_email: commit_info.author_email,
            time: display_time(commit_info.time),
            files: files_raw
                .into_iter()
                .map(|summary| build_commit_file_summary(&ctx, &commit_hash, summary))
                .collect(),
            files_changed: diff_stats.files_changed,
            insertions: diff_stats.insertions,
            deletions: diff_stats.deletions,
        })
    })
}

pub async fn commit_diff(
    Path((repo_name, commit_hash)): Path<(String, String)>,
    Query(query): Query<CommitDiffQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let ctx = RepoRequestContext::load(&state, repo_name, query.ref_name)?;

    if !is_safe_repo_path(&query.path) {
        return Err(AppError::BadRequest);
    }

    render_template(move || {
        let diff = git::get_commit_diff_for_path(&ctx.repo, &commit_hash, &query.path)
            .map_err(|_| AppError::NotFound)?
            .ok_or(AppError::NotFound)?;

        Ok(CommitDiffBodyTemplate { diff })
    })
}

fn build_commit_file_summary(
    ctx: &RepoRequestContext,
    commit_hash: &str,
    summary: git::CommitDiffSummary,
) -> CommitFileSummary {
    let encoded_path = encode_query_value(&summary.path);

    CommitFileSummary {
        file_name: summary.path,
        adds: summary.adds,
        dels: summary.dels,
        is_binary: summary.is_binary,
        is_added: summary.change == git::CommitDiffChange::Added,
        is_deleted: summary.change == git::CommitDiffChange::Deleted,
        empty_label: summary.empty_label.map(str::to_string),
        diff_href: ctx.append_ref(format!(
            "/{}/commit/{}/diff?path={}",
            ctx.repo_name, commit_hash, encoded_path
        )),
    }
}

fn resolve_branch(
    repo: &git2::Repository,
    commit_hash: &str,
    ref_name: Option<&str>,
) -> Option<String> {
    if let Some(ref_name) = ref_name
        .map(str::trim)
        .filter(|name| !name.is_empty() && *name != "HEAD")
    {
        if let Some(branch) = ref_name.strip_prefix("refs/heads/") {
            if repo.find_reference(ref_name).is_ok() {
                return Some(branch.to_string());
            }
        } else if repo
            .find_reference(&format!("refs/heads/{}", ref_name))
            .is_ok()
        {
            return Some(ref_name.to_string());
        }
    }

    git::find_branch_for_commit(repo, commit_hash)
        .ok()
        .flatten()
}

fn encode_query_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());

    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(byte as char)
            }
            _ => {
                let _ = write!(&mut encoded, "%{:02X}", byte);
            }
        }
    }

    encoded
}
