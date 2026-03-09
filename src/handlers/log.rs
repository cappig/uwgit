use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Html;

use crate::git;
use crate::templates::{CommitDisplay, CommitRefDisplay, LogTemplate};

use super::util::{
    PAGE_SIZE, RepoRequestContext, build_pager, display_time, render_cached_template, run_blocking,
};
use super::{AppError, AppState, LogQuery};

pub async fn log(
    Path(repo_name): Path<String>,
    Query(query): Query<LogQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let offset = page.saturating_sub(1) * PAGE_SIZE;
    run_blocking(move || {
        let ctx = RepoRequestContext::load(&state, repo_name, query.ref_name)?;
        let chrome = ctx.chrome.clone();
        let nav = ctx.nav("log");
        let base = format!("/{}/log", ctx.repo_name);
        let cache_key = format!(
            "log:{}:{}:{}:{}",
            ctx.repo_name,
            ctx.display_ref,
            ctx.commit_oid()?,
            page
        );

        render_cached_template(&state.short_html_cache, cache_key, move || {
            let (commits_raw, total_commits) =
                git::get_commits_paginated(&ctx.repo, ctx.git_ref(), offset, PAGE_SIZE)
                    .map_err(|_| AppError::BadRequest)?;
            let total_pages = super::util::total_pages(total_commits, PAGE_SIZE);
            let commit_hashes = commits_raw
                .iter()
                .map(|c| c.hash.clone())
                .collect::<Vec<_>>();
            let refs_by_commit =
                git::refs_for_commits(&ctx.repo, &commit_hashes).unwrap_or_default();

            let commits = commits_raw
                .into_iter()
                .map(|c| {
                    let refs = refs_by_commit
                        .get(&c.hash)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|r| CommitRefDisplay {
                            name: r.name,
                            class: if r.kind == "branch" {
                                "commit-ref commit-branch"
                            } else {
                                "commit-ref commit-tag"
                            },
                        })
                        .collect();

                    CommitDisplay {
                        href: ctx.append_ref(format!("/{}/commit/{}", ctx.repo_name, c.hash)),
                        hash: c.hash,
                        short_hash: c.short_hash,
                        author: c.author,
                        message: c.message.lines().next().unwrap_or("").to_string(),
                        time: display_time(c.time),
                        refs,
                    }
                })
                .collect();

            Ok(LogTemplate {
                chrome,
                nav,
                commits,
                pager: build_pager(&base, &ctx.display_ref, page, total_pages),
            })
        })
    })
    .await
}
