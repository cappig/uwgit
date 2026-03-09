mod config;
mod format;
mod git;
mod handlers;
mod highlight;
mod templates;

use axum::{Router, http::StatusCode, routing::get};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::config::AppConfig;

#[tokio::main]
async fn main() {
    let config = AppConfig::load().expect("failed to load config");
    let app = build_app(&config);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

fn build_app(config: &AppConfig) -> Router {
    handlers::errors::set_site_title(config.site_title.clone());

    Router::new()
        .route("/", get(handlers::list_repos))
        .route("/favicon.ico", get(|| async { StatusCode::NO_CONTENT }))
        .route("/{repo}", get(handlers::index))
        .route("/{repo}/archive.tar.gz", get(handlers::archive))
        .route("/{repo}/refs", get(handlers::refs))
        .route("/{repo}/tree", get(handlers::tree))
        .route("/{repo}/log", get(handlers::log))
        .route("/{repo}/blob", get(handlers::blob))
        .route("/{repo}/commit/{hash}/diff", get(handlers::commit_diff))
        .route("/{repo}/commit/{hash}", get(handlers::commit))
        .nest_service("/static", ServeDir::new("static"))
        .fallback(handlers::errors::not_found)
        .with_state(Arc::new(handlers::AppState::from_config(config)))
}
