use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Html;

use crate::git;
use crate::templates::{RefDisplay, RefsTemplate};

use super::util::{
    PAGE_SIZE, RepoRequestContext, append_ref, build_pager, display_time, render_cached_template,
    run_blocking,
};
use super::{AppError, AppState, LogQuery};

pub async fn refs(
    Path(repo_name): Path<String>,
    Query(query): Query<LogQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let offset = page.saturating_sub(1) * PAGE_SIZE;
    run_blocking(move || {
        let ctx = RepoRequestContext::load(&state, repo_name, query.ref_name)?;
        let chrome = ctx.chrome.clone();
        let nav = ctx.nav("refs");
        let base = format!("/{}/refs", ctx.repo_name);
        let cache_key = format!("refs:{}:{}:{}", ctx.repo_name, ctx.display_ref, page);

        render_cached_template(&state.short_html_cache, cache_key, move || {
            let (refs_raw, total_refs) = git::list_refs_paginated(&ctx.repo, offset, PAGE_SIZE)?;
            let total_pages = super::util::total_pages(total_refs, PAGE_SIZE);
            let refs = refs_raw
                .into_iter()
                .map(|r| RefDisplay {
                    href: append_ref(format!("/{}/log", ctx.repo_name), &r.name),
                    name: r.name,
                    ref_type: r.ref_type,
                    author: r.author,
                    time: display_time(r.time),
                })
                .collect();

            Ok(RefsTemplate {
                chrome,
                nav,
                refs,
                pager: build_pager(&base, &ctx.display_ref, page, total_pages),
            })
        })
    })
    .await
}
