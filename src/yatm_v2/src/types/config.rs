use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub yatm_v2_version: String,
    pub repo_name: String,
    pub repo_owner: String,
    pub requirements_dirs: Vec<PathBuf>,
    pub test_cases_builders_dirs: Vec<PathBuf>,
    pub new_requirements_dir: PathBuf,
    pub new_test_cases_builder_dir: PathBuf,
    pub generated_files_dir: PathBuf,
}

impl Config {
    pub fn default() -> Self {
        let version: &str = env!("CARGO_PKG_VERSION");
        let requirements_dir = "requirements";
        let test_cases_builder_dir = "test_cases_builders";
        Config {
            yatm_v2_version: version.to_string(),
            repo_name: "repo_name".to_string(),
            repo_owner: "repo_owner".to_string(),
            requirements_dirs: vec![PathBuf::new().join(requirements_dir)],
            new_requirements_dir: PathBuf::new().join(requirements_dir),
            test_cases_builders_dirs: vec![PathBuf::new().join(test_cases_builder_dir)],
            new_test_cases_builder_dir: PathBuf::new().join(test_cases_builder_dir),
            generated_files_dir: PathBuf::new().join("generated_files"),
        }
    }
}
