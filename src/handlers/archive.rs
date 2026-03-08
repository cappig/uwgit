use std::process::Command;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Response};

use crate::git;

use super::util::RepoRequestContext;
use super::{AppError, AppState, ArchiveQuery};

pub async fn archive(
    Path(repo_name): Path<String>,
    Query(query): Query<ArchiveQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let ctx = RepoRequestContext::load(&state, repo_name, query.ref_name)?;
    let archive_root = archive_root_name(&ctx);
    let filename = format!("{archive_root}.tar.gz");
    let (git_dir, rev) = {
        let commit =
            git::commit_for_ref(&ctx.repo, ctx.git_ref()).map_err(|_| AppError::NotFound)?;
        (ctx.repo.path().to_path_buf(), commit.id().to_string())
    };
    drop(ctx);

    let bytes = tokio::task::spawn_blocking(move || build_archive(&git_dir, &rev, &archive_root))
        .await
        .map_err(|err| AppError::Internal(err.into()))??;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/gzip"),
    );

    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }

    Ok((headers, bytes).into_response())
}

fn build_archive(
    git_dir: &std::path::Path,
    rev: &str,
    archive_root: &str,
) -> Result<Vec<u8>, AppError> {
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(git_dir)
        .arg("archive")
        .arg("--format=tar.gz")
        .arg(format!("--prefix={archive_root}/"))
        .arg(rev)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git archive failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }

    Ok(output.stdout)
}

fn archive_root_name(ctx: &RepoRequestContext) -> String {
    let ref_name = if ctx.display_ref.is_empty() {
        ctx.chrome.header_branch.as_deref().unwrap_or("HEAD")
    } else {
        pretty_ref_name(&ctx.display_ref)
    };

    format!(
        "{}-{}",
        sanitize_archive_component(&ctx.repo_name),
        sanitize_archive_component(ref_name)
    )
}

fn pretty_ref_name(ref_name: &str) -> &str {
    ref_name
        .strip_prefix("refs/heads/")
        .or_else(|| ref_name.strip_prefix("refs/tags/"))
        .unwrap_or(ref_name)
}

fn sanitize_archive_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => ch,
            _ => '-',
        })
        .collect();

    sanitized.trim_matches('-').to_string()
}
