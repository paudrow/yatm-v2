use crate::constants::YAML_EXTENSIONS;
use crate::helpers::get_files;
use anyhow::{Context, Result};
use common::types::Requirement;
use common::types::RequirementsFile;
use serde_yaml;
use std::path::PathBuf;

/// Validate the requirements files.
pub fn validate_requirements_files(requirement_dirs: &Vec<PathBuf>) -> Result<()> {
    get_requirements_from_files(requirement_dirs)?;
    Ok(())
}

/// Validate the requirements file.
pub fn validate_requirements_file(requirement_path: &PathBuf) -> Result<()> {
    get_requirements_from_file(requirement_path)?;
    Ok(())
}

#[cfg(test)]
mod test_validate_requirement {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_validate_requirement() {
        let dir = tempdir().unwrap();
        let requirement_path = dir.path().join("requirement.yaml");
        let mut file = File::create(&requirement_path).unwrap();
        let requirements_file = RequirementsFile::default();
        let requirements_file = serde_yaml::to_string(&requirements_file).unwrap();
        file.write_all(requirements_file.as_bytes()).unwrap();
        validate_requirements_file(&requirement_path).unwrap();
    }

    #[test]
    fn test_validate_requirement_invalid() {
        let dir = tempdir().unwrap();
        let requirement_path = dir.path().join("requirement.yaml");
        let mut file = File::create(&requirement_path).unwrap();
        let requirement_str = "invalid";
        file.write_all(requirement_str.as_bytes()).unwrap();
        assert!(validate_requirements_file(&requirement_path).is_err());
    }
}

/// Get the requirements from the files.
pub fn get_requirements_from_files(requirement_dirs: &Vec<PathBuf>) -> Result<Vec<Requirement>> {
    let requirement_files = get_files(&requirement_dirs, &YAML_EXTENSIONS).context(format!(
        "Failed to get the requirement files: {:?}",
        requirement_dirs
    ))?;
    let mut all_requirements: Vec<Requirement> = vec![];
    for requirement_file in requirement_files {
        let requirements = get_requirements_from_file(&requirement_file).context(format!(
            "Failed to validate the requirement: {:?}",
            requirement_file
        ))?;
        all_requirements.extend(requirements);
    }
    Ok(all_requirements)
}

/// Get the requirements from a file.
pub fn get_requirements_from_file(requirement_path: &PathBuf) -> Result<Vec<Requirement>> {
    let requirement = std::fs::read_to_string(&requirement_path).context(format!(
        "Failed to read the requirement file: {:?}",
        requirement_path
    ))?;
    let requirements_file =
        serde_yaml::from_str::<RequirementsFile>(&requirement).context(format!(
            "Failed to deserialize the requirement: {:?}",
            requirement_path
        ))?;
    Ok(requirements_file.requirements)
}
