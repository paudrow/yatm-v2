use common::types::GithubLabel;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// The owner of the repository.
    pub repo_owner: String,
    /// The name of the repository.
    pub repo_name: String,
    /// The version of the workspace.
    pub workspace_version: String,
    /// The version of yatm_v2 to use.
    pub yatm_v2_version: String,
    /// The directories to search for requirements.
    pub requirements_dirs: Vec<PathBuf>,
    /// The directories to search for test cases builders.
    pub test_cases_builders_dirs: Vec<PathBuf>,
    /// The directory to create new requirements.
    pub new_requirements_dir: PathBuf,
    /// The directory to create new test cases builders.
    pub new_test_cases_builder_dir: PathBuf,
    /// The directory to store generated files.
    pub generated_files_dir: PathBuf,
    /// The labels to create in the repository.
    pub labels: Vec<GithubLabel>,
}

impl Config {
    pub fn default() -> Self {
        let version: &str = env!("CARGO_PKG_VERSION");
        let requirements_dir = "requirements";
        let test_cases_builder_dir = "test_cases_builders";
        Config {
            workspace_version: "0.0.1".to_string(),
            yatm_v2_version: version.to_string(),
            repo_name: "repo_name".to_string(),
            repo_owner: "repo_owner".to_string(),
            requirements_dirs: vec![PathBuf::new().join(requirements_dir)],
            new_requirements_dir: PathBuf::new().join(requirements_dir),
            test_cases_builders_dirs: vec![PathBuf::new().join(test_cases_builder_dir)],
            new_test_cases_builder_dir: PathBuf::new().join(test_cases_builder_dir),
            generated_files_dir: PathBuf::new().join("generated_files"),
            labels: vec![
                GithubLabel {
                    name: "needs attention: bug".to_string(),
                    color: "f0440a".to_string(),
                    description: Some("A bug has been found and needs to be confirmed".to_string()),
                },
                GithubLabel {
                    name: "needs attention: bad instructions".to_string(),
                    color: "f0440a".to_string(),
                    description: Some(
                        "The issue instructions don't appear to be correct or complete".to_string(),
                    ),
                },
                GithubLabel {
                    name: "confirmed: bug".to_string(),
                    color: "dcacf2".to_string(),
                    description: Some("A bug was confirmed".to_string()),
                },
                GithubLabel {
                    name: "confirmed: bad instructions".to_string(),
                    color: "dcacf2".to_string(),
                    description: Some(
                        "The issue instructions have been confirmed to be incorrect or incomplete"
                            .to_string(),
                    ),
                },
                GithubLabel {
                    name: "confirmed: works as expected".to_string(),
                    color: "cef2ac".to_string(),
                    description: Some("Works as expected".to_string()),
                },
                GithubLabel {
                    name: "who: community tested".to_string(),
                    color: "b1e8fd".to_string(),
                    description: Some("A community member has tested this".to_string()),
                },
                GithubLabel {
                    name: "who: core team tested".to_string(),
                    color: "55d0db".to_string(),
                    description: Some("A core team member has tested this".to_string()),
                },
            ],
        }
    }
}
