use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub yatm_v2_version: String,
    pub repo_name: String,
    pub repo_owner: String,
    pub requirements_dirs: Vec<PathBuf>,
    pub generated_files_dir: PathBuf,
}

impl Config {
    pub fn default() -> Self {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        Config {
            yatm_v2_version: VERSION.to_string(),
            repo_name: "repo_name".to_string(),
            repo_owner: "repo_owner".to_string(),
            requirements_dirs: vec![PathBuf::new().join("requirements")],
            generated_files_dir: PathBuf::new().join(".generated_files"),
        }
    }
}
