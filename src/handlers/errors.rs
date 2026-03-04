use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::sync::{Arc, OnceLock};

use super::AppState;
use super::util::site_chrome;
use crate::templates::ErrorTemplate;

pub enum AppError {
    BadRequest,
    NotFound,
    Internal(anyhow::Error),
}

static SITE_TITLE: OnceLock<String> = OnceLock::new();

pub fn set_site_title(title: String) {
    let _ = SITE_TITLE.set(title);
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        return match self {
            AppError::BadRequest => fallback(
                None,
                StatusCode::BAD_REQUEST,
                "Bad request",
                "The request was invalid or malformed.",
            ),
            AppError::NotFound => fallback(
                None,
                StatusCode::NOT_FOUND,
                "Not found",
                "The page you requested does not exist.",
            ),
            AppError::Internal(err) => {
                eprintln!("internal error: {err:?}");
                fallback(
                    None,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal error",
                    "An unexpected error occurred.",
                )
            }
        };
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        return Self::Internal(err.into());
    }
}

fn fallback(
    header_repo: Option<String>,
    status: StatusCode,
    heading: &str,
    message: &str,
) -> Response {
    let tpl = ErrorTemplate {
        chrome: crate::templates::PageChrome {
            header_repo,
            ..site_chrome(site_title())
        },
        heading: heading.to_string(),
        message: message.to_string(),
    };

    return match tpl.render() {
        Ok(html) => (status, axum::response::Html(html)).into_response(),
        Err(_) => (status, heading.to_string()).into_response(),
    };
}

pub async fn not_found(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = SITE_TITLE.set(state.site_title.clone());

    return fallback(
        None,
        StatusCode::NOT_FOUND,
        "Not found",
        "The page you requested does not exist.",
    );
}

fn site_title() -> String {
    return SITE_TITLE
        .get()
        .cloned()
        .unwrap_or_else(|| "uwgit".to_string());
}
