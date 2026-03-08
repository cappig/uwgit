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
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();

        Ok(Self {
            repos_path: required_var("REPOS_PATH")?,
            site_title: required_var("SITE_TITLE")?,
            owner: required_var("OWNER")?,
            host: required_var("HOST")?,
            port: required_var("PORT")?
                .parse()
                .with_context(|| "failed to parse PORT as an integer".to_string())?,
        })
    }
}

fn required_var(name: &str) -> anyhow::Result<String> {
    std::env::var(name).with_context(|| format!("missing required env var: {}", name))
}
