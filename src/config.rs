use std::{env, fs, path::PathBuf};

use serde::Deserialize;

use crate::error::Error;

// Default configuration file.
const DEFAULT_CONFIG_PATH: &str = "config.toml";

// Environment variable to specify configuration file.
const CONFIG_ENV_VAR: &str = "RITE_CONFIG";

#[derive(Deserialize)]
pub struct Config {
    pub url: String,
    pub title: String,

    pub content: PathBuf,
    pub posts: PathBuf,

    pub build_root: PathBuf,
    pub posts_root: PathBuf,

    pub syntax_theme: Option<PathBuf>,
    pub posts_src_scripts: Option<Vec<String>>,
    pub posts_embed_scripts: Option<PathBuf>,
    pub posts_noscript: Option<String>,
}

/// Read the configuration file.
/// Defaults to `config.toml` unless overridden by the `RITE_CONFIG`
/// environment variable.
pub fn read_config() -> Result<Config, Error> {
    let path: PathBuf = env::var(CONFIG_ENV_VAR)
        .unwrap_or(DEFAULT_CONFIG_PATH.into())
        .into();

    if !path.exists() {
        return Err(Error::MissingConfig(path));
    }

    let contents = fs::read_to_string(&path)?;
    let mut config: Config = toml::from_str(&contents).map_err(|e| Error::ReadConfig(path, e))?;
    config.url = config.url.trim_end_matches('/').to_string();
    Ok(config)
}
