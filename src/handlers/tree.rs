use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Html;

use crate::format::human_readable_size;
use crate::git;
use crate::templates::{TreeEntryDisplay, TreeTemplate};

use super::util::{RepoRequestContext, display_time, is_safe_repo_path, render_template};
use super::{AppError, AppState, TreeQuery};

pub async fn tree(
    Path(repo_name): Path<String>,
    Query(query): Query<TreeQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let ctx = RepoRequestContext::load(&state, repo_name, query.ref_name)?;
    let path = query.path.unwrap_or_default();
    if !is_safe_repo_path(&path) {
        return Err(AppError::BadRequest);
    }

    let parent_path = if path.is_empty() {
        String::new()
    } else {
        path.rsplit_once('/')
            .map(|x| x.0.to_string())
            .unwrap_or_default()
    };

    let chrome = ctx.chrome.clone();
    let nav = ctx.nav("tree");
    let root_href = ctx.append_ref(format!("/{}/tree", ctx.repo_name));
    let path_components = ctx.path_components(&path, true);

    render_template(move || {
        let entries = git::get_tree_entries(&ctx.repo, ctx.git_ref(), &path)
            .map_err(|_| AppError::NotFound)?
            .into_iter()
            .map(|entry| {
                let entry_path = if path.is_empty() {
                    entry.name.clone()
                } else {
                    format!("{}/{}", path, entry.name)
                };
                let size_text = if entry.is_dir {
                    entry
                        .size
                        .map(|size| format!("{} entries", size))
                        .unwrap_or_default()
                } else {
                    entry.size.map(human_readable_size).unwrap_or_default()
                };

                TreeEntryDisplay {
                    href: ctx.append_ref(if entry.is_dir {
                        format!("/{}/tree?path={}", ctx.repo_name, entry_path)
                    } else {
                        format!("/{}/blob?path={}", ctx.repo_name, entry_path)
                    }),
                    name: entry.name,
                    is_dir: entry.is_dir,
                    size_text,
                    time: entry.last_modified.map(display_time),
                }
            })
            .collect();

        Ok(TreeTemplate {
            chrome,
            nav,
            entries,
            path: path.clone(),
            parent_href: if parent_path.is_empty() {
                root_href.clone()
            } else {
                ctx.append_ref(format!("/{}/tree?path={}", ctx.repo_name, parent_path))
            },
            root_href,
            path_components,
        })
    })
}
