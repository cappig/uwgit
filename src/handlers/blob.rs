use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Response};

use crate::git;
use crate::highlight;
use crate::templates::{BlobTemplate, DiffLine};

use super::util::{
    RepoRequestContext, content_type_for_extension, is_safe_repo_path, render_cached_template,
    run_blocking,
};
use super::{AppError, AppState, BlobQuery};

pub async fn blob(
    Path(repo_name): Path<String>,
    Query(query): Query<BlobQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    if !is_safe_repo_path(&query.path) {
        return Err(AppError::BadRequest);
    }

    let is_raw = query.raw == Some(1);
    run_blocking(move || {
        let ctx = RepoRequestContext::load(&state, repo_name, query.ref_name)?;
        let content_bytes = git::get_file_content(&ctx.repo, ctx.git_ref(), &query.path)
            .map_err(|_| AppError::NotFound)?;

        let ext = std::path::Path::new(&query.path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let content_type = content_type_for_extension(&ext);
        let is_binary = git::is_binary_bytes(&content_bytes);

        if is_raw {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));

            if is_binary {
                let filename = std::path::Path::new(&query.path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("blob");

                let sanitized = filename.replace('"', "_");

                if let Ok(value) =
                    HeaderValue::from_str(&format!("attachment; filename=\"{}\"", sanitized))
                {
                    headers.insert(header::CONTENT_DISPOSITION, value);
                }
            }

            return Ok((headers, content_bytes).into_response());
        }

        let chrome = ctx.chrome.clone();
        let nav = ctx.nav("blob");
        let is_image = content_type.starts_with("image/");
        let cache_key = format!(
            "blob:{}:{}:{}:{}",
            ctx.repo_name,
            ctx.display_ref,
            ctx.commit_oid()?,
            query.path
        );

        let html = render_cached_template(&state.long_html_cache, cache_key, move || {
            let lines = if is_binary {
                Vec::new()
            } else {
                let raw_lines: Vec<String> = String::from_utf8_lossy(&content_bytes)
                    .split_terminator('\n')
                    .map(str::to_string)
                    .collect();
                let highlighted = highlight::highlight_lines(
                    &query.path,
                    raw_lines.iter().map(String::as_str),
                    true,
                );

                raw_lines
                    .into_iter()
                    .zip(highlighted)
                    .enumerate()
                    .map(|(idx, (_text, html))| DiffLine {
                        class: "",
                        num: (idx + 1).to_string(),
                        old_num: None,
                        new_num: None,
                        html,
                    })
                    .collect()
            };

            Ok(BlobTemplate {
                chrome,
                nav,
                lines,
                blob_href: ctx.append_ref(format!("/{}/blob?path={}", ctx.repo_name, query.path)),
                root_href: ctx.append_ref(format!("/{}/tree", ctx.repo_name)),
                is_binary,
                is_image,
                path_components: ctx.path_components(&query.path, false),
                file_path: query.path,
            })
        })?;

        Ok(html.into_response())
    })
    .await
}
