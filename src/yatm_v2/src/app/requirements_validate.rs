use crate::types::RequirementsFile;
use anyhow::{Context, Result};
use common::types::Requirement;
use serde_yaml;
use std::path::PathBuf;

pub fn validate_requirements_files(requirement_dirs: &Vec<PathBuf>) -> Result<()> {
    get_requirements_from_files(requirement_dirs)?;
    Ok(())
}

pub fn validate_requirements_file(requirement_path: &PathBuf) -> Result<()> {
    get_requirements_from_file(requirement_path)?;
    Ok(())
}

pub fn get_requirements_from_files(requirement_dirs: &Vec<PathBuf>) -> Result<Vec<Requirement>> {
    let requirement_files = get_requirements_files(&requirement_dirs).context(format!(
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

pub fn get_requirements_files(requirement_dirs: &Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut requirements_files: Vec<PathBuf> = vec![];
    for requirement_dir in requirement_dirs {
        let requirement_files = std::fs::read_dir(&requirement_dir).context(format!(
            "Failed to read the requirement directory: {:?}",
            requirement_dir
        ))?;
        for requirement_file in requirement_files {
            let requirement_file = requirement_file.context(format!(
                "Failed to read the entry in the requirement directory: {:?}",
                requirement_dir
            ))?;
            let requirement_path = requirement_file.path();
            requirements_files.push(requirement_path);
        }
    }
    if requirements_files.is_empty() {
        return Err(anyhow::anyhow!(format!(
            "No requirement files found: {:#?}",
            requirement_dirs
        )));
    }
    Ok(requirements_files)
}

#[cfg(test)]
mod test_validate_requirement {
    use super::*;
    use common::types::Requirement;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_validate_requirement() {
        let dir = tempdir().unwrap();
        let requirement_path = dir.path().join("requirement.yaml");
        let mut file = File::create(&requirement_path).unwrap();
        let requirement = Requirement {
            name: "name".to_string(),
            description: "description".to_string(),
            steps: vec![],
            labels: None,
            links: None,
        };
        let requirements_file = RequirementsFile {
            requirements: vec![requirement.clone()],
        };
        let requirement_file = serde_yaml::to_string(&requirements_file).unwrap();
        file.write_all(requirement_file.as_bytes()).unwrap();
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
