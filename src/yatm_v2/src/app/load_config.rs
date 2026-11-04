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

    if let Some(settings) = &config.repo_settings {
        for collaborator in &settings.collaborators {
            if collaborator.username.trim().is_empty() {
                anyhow::bail!(
                    "Collaborator username cannot be empty in config: {:?}",
                    path
                );
            }
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::types::CollaboratorPermission;
    use tempfile::tempdir;

    #[test]
    fn test_load_legacy_config_without_repo_settings() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config_test.yaml");
        let yaml = r#"
repo_owner: intrinsic-checklists
repo_name: test-onboarding
workspace_version: 0.0.1
yatm_v2_version: 1.0.0
minimum_permutations_to_render: 1
requirements_dirs:
- requirements
test_cases_builders_dirs:
- onboarding_default_tasks
new_requirements_dir: requirements
new_test_cases_builder_dir: test_cases_builders
generated_files_dir: generated_files
labels: []
"#;
        std::fs::write(&config_path, yaml).unwrap();
        let config = load_config(&config_path).unwrap();
        assert!(config.repo_settings.is_none());
    }

    #[test]
    fn test_load_config_with_repo_settings_and_collaborators() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config_test.yaml");
        let yaml = r#"
repo_owner: intrinsic-checklists
repo_name: test-onboarding
repo_settings:
  create_if_missing: true
  private: true
  description: Onboarding checklist for testuser
  strict_collaborators: true
  collaborators:
  - username: manager1
    permission: admin
  - username: report1
    permission: maintain
workspace_version: 0.0.1
yatm_v2_version: 1.0.0
requirements_dirs:
- requirements
test_cases_builders_dirs:
- onboarding_default_tasks
new_requirements_dir: requirements
new_test_cases_builder_dir: test_cases_builders
generated_files_dir: generated_files
labels: []
"#;
        std::fs::write(&config_path, yaml).unwrap();
        let config = load_config(&config_path).unwrap();
        let settings = config.repo_settings.unwrap();
        assert!(settings.create_if_missing);
        assert!(settings.reapply_settings_after_creation);
        assert!(settings.private);
        assert!(settings.strict_collaborators);
        assert_eq!(
            settings.description.as_deref(),
            Some("Onboarding checklist for testuser")
        );
        assert_eq!(settings.collaborators.len(), 2);
        assert_eq!(settings.collaborators[0].username, "manager1");
        assert_eq!(
            settings.collaborators[0].permission,
            CollaboratorPermission::Admin
        );
        assert_eq!(settings.collaborators[1].username, "report1");
        assert_eq!(
            settings.collaborators[1].permission,
            CollaboratorPermission::Maintain
        );

        // Also verify reapply_settings_after_creation can be explicitly set to false
        let no_reapply_yaml = yaml.replace(
            "  create_if_missing: true",
            "  create_if_missing: true\n  reapply_settings_after_creation: false",
        );
        std::fs::write(&config_path, no_reapply_yaml).unwrap();
        let config_no_reapply = load_config(&config_path).unwrap();
        assert!(
            !config_no_reapply
                .repo_settings
                .unwrap()
                .reapply_settings_after_creation
        );
    }

    #[test]
    fn test_load_config_rejects_invalid_permission_or_empty_username() {
        let dir = tempdir().unwrap();
        let bad_perm_path = dir.path().join("bad_perm.yaml");
        let bad_perm_yaml = r#"
repo_owner: intrinsic-checklists
repo_name: test-onboarding
repo_settings:
  collaborators:
  - username: manager1
    permission: superowner
workspace_version: 0.0.1
yatm_v2_version: 1.0.0
requirements_dirs: [requirements]
test_cases_builders_dirs: [test_cases_builders]
new_requirements_dir: requirements
new_test_cases_builder_dir: test_cases_builders
generated_files_dir: generated_files
labels: []
"#;
        std::fs::write(&bad_perm_path, bad_perm_yaml).unwrap();
        assert!(load_config(&bad_perm_path).is_err());

        let empty_user_path = dir.path().join("empty_user.yaml");
        let empty_user_yaml = r#"
repo_owner: intrinsic-checklists
repo_name: test-onboarding
repo_settings:
  collaborators:
  - username: "   "
    permission: admin
workspace_version: 0.0.1
yatm_v2_version: 1.0.0
requirements_dirs: [requirements]
test_cases_builders_dirs: [test_cases_builders]
new_requirements_dir: requirements
new_test_cases_builder_dir: test_cases_builders
generated_files_dir: generated_files
labels: []
"#;
        std::fs::write(&empty_user_path, empty_user_yaml).unwrap();
        assert!(load_config(&empty_user_path).is_err());
    }
}
