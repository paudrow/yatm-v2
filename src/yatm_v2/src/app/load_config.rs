use crate::types::Config;
use anyhow::{Context, Result};
use serde_yaml;
use std::path::PathBuf;

/// Load the configuration.
pub fn load_config(path: &PathBuf) -> Result<Config> {
    let mut path = path.clone();
    if path.is_dir() {
        path = path.join("config.yaml".to_string());
    }

    let config =
        std::fs::read_to_string(&path).context(format!("No config file found: {:?}", path))?;
    let mut config = serde_yaml::from_str::<Config>(&config)
        .context(format!("Failed to deserialize the config: {:?}", path))?;

    // Update the paths to be relative to the config file
    let parent_dir = path
        .parent()
        .context(format!("Failed to get the parent directory: {:?}", path))?;
    config.requirements_dirs = config
        .requirements_dirs
        .iter()
        .map(|dir| parent_dir.join(dir))
        .collect();
    config.new_requirements_dir = parent_dir.join(config.new_requirements_dir);
    config.test_cases_builders_dirs = config
        .test_cases_builders_dirs
        .iter()
        .map(|file| parent_dir.join(file))
        .collect();
    config.new_test_cases_builder_dir = parent_dir.join(config.new_test_cases_builder_dir);
    config.generated_files_dir = parent_dir.join(config.generated_files_dir);

    Ok(config)
}
