use crate::config::AppConfig;

pub mod errors;
pub mod util;

mod blob;
mod commit;
mod log;
mod refs;
mod repo;
mod tree;

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
}

impl AppState {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            repos_path: config.repos_path.clone().into(),
            site_title: config.site_title.clone(),
            owner: config.owner.clone(),
        }
    }
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
