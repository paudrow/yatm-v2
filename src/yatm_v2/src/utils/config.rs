use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub repo_name: String,
    pub repo_owner: String,
    pub requirements_dirs: Vec<PathBuf>,
    pub generated_files_dir: PathBuf,
}

impl Config {
    pub fn default() -> Self {
        Config {
            repo_name: "repo_name".to_string(),
            repo_owner: "repo_owner".to_string(),
            requirements_dirs: vec![PathBuf::new().join("requirements")],
            generated_files_dir: PathBuf::new(),
        }
    }
}
