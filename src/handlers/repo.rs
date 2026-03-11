use std::collections::HashSet;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Html;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

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

        let repo_name = ctx.repo_name.clone();
        let display_ref = ctx.display_ref.clone();
        render_cached_template(&state.long_html_cache, cache_key, move || {
            Ok(IndexTemplate {
                chrome,
                nav,
                readme: git::get_readme(&ctx.repo, ctx.git_ref())?
                    .map(|r| render_readme(r, &repo_name, &display_ref)),
            })
        })
    })
    .await
}

fn render_readme(readme: git::ReadmeContent, repo_name: &str, display_ref: &str) -> ReadmeDisplay {
    let content = if readme.is_markdown {
        render_markdown(&readme.text, repo_name, display_ref)
    } else {
        readme.text
    };

    ReadmeDisplay {
        content,
        is_markdown: readme.is_markdown,
    }
}

fn is_relative_url(url: &str) -> bool {
    !url.starts_with("http://")
        && !url.starts_with("https://")
        && !url.starts_with("//")
        && !url.starts_with("data:")
        && !url.starts_with('#')
}

fn rewrite_image_url(url: &str, repo_name: &str, display_ref: &str) -> String {
    let path = url.trim_start_matches("./");
    let mut raw_url = format!(
        "/{}/blob?path={}&raw=1",
        repo_name,
        urlencoding::encode(path)
    );

    if !display_ref.is_empty() {
        raw_url.push_str("&ref=");
        raw_url.push_str(&urlencoding::encode(display_ref));
    }

    raw_url
}

fn render_markdown(source: &str, repo_name: &str, display_ref: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(source, options).map(|event| match event {
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let dest_url = if is_relative_url(&dest_url) {
                CowStr::from(rewrite_image_url(&dest_url, repo_name, display_ref))
            } else {
                dest_url
            };
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            })
        }
        other => other,
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let mut allowed_tags = ammonia::Builder::default().clone_tags();
    allowed_tags.insert("img");

    let mut tag_attrs = ammonia::Builder::default().clone_tag_attributes();
    let img_attrs: HashSet<&str> = ["src", "alt", "title"].into();
    tag_attrs.insert("img", img_attrs);

    let mut builder = ammonia::Builder::default();
    builder.tags(allowed_tags).tag_attributes(tag_attrs);
    builder.clean(&html_output).to_string()
}
