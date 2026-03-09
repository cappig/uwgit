use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Html;
use pulldown_cmark::{Options, Parser, html};

use crate::git;
use crate::templates::{IndexTemplate, ReadmeDisplay, ReposTemplate};

use super::util::{RepoRequestContext, render_cached_template, run_blocking, site_chrome};
use super::{AppError, AppState, LogQuery};

pub async fn list_repos(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    run_blocking(move || {
        render_cached_template(&state.short_html_cache, "repos".to_string(), || {
            Ok(ReposTemplate {
                repos: git::list_repos(&state.repos_path)?,
                owner: state.owner.clone(),
                chrome: site_chrome(state.site_title.clone()),
            })
        })
    })
    .await
}

pub async fn index(
    Path(repo_name): Path<String>,
    Query(query): Query<LogQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    run_blocking(move || {
        let ctx = RepoRequestContext::load(&state, repo_name, query.ref_name)?;
        let chrome = ctx.chrome.clone();
        let nav = ctx.nav("index");
        let cache_key = format!(
            "index:{}:{}:{}",
            ctx.repo_name,
            ctx.display_ref,
            ctx.commit_oid()?
        );

        render_cached_template(&state.long_html_cache, cache_key, move || {
            Ok(IndexTemplate {
                chrome,
                nav,
                readme: git::get_readme(&ctx.repo, ctx.git_ref())?.map(render_readme),
            })
        })
    })
    .await
}

fn render_readme(readme: git::ReadmeContent) -> ReadmeDisplay {
    let content = if readme.is_markdown {
        render_markdown(&readme.text)
    } else {
        readme.text
    };

    ReadmeDisplay {
        content,
        is_markdown: readme.is_markdown,
    }
}

fn render_markdown(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(source, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    ammonia::clean(&html_output)
}
