use crate::config::AppConfig;
use moka::sync::Cache;
use std::time::Duration;

pub mod errors;
pub mod util;

mod archive;
mod blob;
mod commit;
mod log;
mod refs;
mod repo;
mod tree;

pub use archive::archive;
pub use blob::blob;
pub use commit::{commit, commit_diff};
pub use errors::AppError;
pub use log::log;
pub use refs::refs;
pub use repo::{index, list_repos};
pub use tree::tree;

pub struct AppState {
    pub repos_path: std::path::PathBuf,
    pub site_title: String,
    pub owner: String,
    pub short_html_cache: Cache<String, String>,
    pub long_html_cache: Cache<String, String>,
}

impl AppState {
    pub fn from_config(config: &AppConfig) -> anyhow::Result<Self> {
        let repos_path = std::fs::canonicalize(&config.repos_path)?;

        Ok(Self {
            repos_path,
            site_title: config.site_title.clone(),
            owner: config.owner.clone(),
            short_html_cache: Cache::builder()
                .max_capacity(128)
                .time_to_live(Duration::from_secs(10))
                .build(),
            long_html_cache: Cache::builder()
                .max_capacity(1024)
                .time_to_live(Duration::from_secs(60 * 30))
                .build(),
        })
    }
}

#[derive(serde::Deserialize)]
pub struct ArchiveQuery {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct TreeQuery {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub path: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct BlobQuery {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub path: String,
    pub raw: Option<u8>,
}

#[derive(serde::Deserialize)]
pub struct LogQuery {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub page: Option<usize>,
}

#[derive(serde::Deserialize)]
pub struct CommitQuery {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CommitDiffQuery {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub path: String,
}
