use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub repos_path: String,
    pub site_title: String,
    pub owner: String,
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config: {}", path.display()))?;

        return toml::from_str(&raw)
            .with_context(|| format!("failed to parse config: {}", path.display()));
    }
}
