use crate::types::Config;
use anyhow::{Context, Result};
use serde_yaml;
use std::path::PathBuf;

pub fn load_config(path: &PathBuf) -> Result<Config> {
    let mut path = path.clone();
    if path.is_dir() {
        path = path.join("config.yaml".to_string());
    }

    let config = std::fs::read_to_string(&path)
        .context(format!("Failed to read the config file: {:?}", path))?;
    let mut config =
        serde_yaml::from_str::<Config>(&config).context("Failed to deserialize the config")?;

    let parent_dir = path
        .parent()
        .context("Failed to get the parent directory")?;
    config.generated_files_dir = parent_dir.join(config.generated_files_dir);
    config.requirements_dirs = config
        .requirements_dirs
        .iter()
        .map(|dir| parent_dir.join(dir))
        .collect();

    Ok(config)
}
