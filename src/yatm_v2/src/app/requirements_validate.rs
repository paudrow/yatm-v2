use anyhow::{Context, Result};
use common::types::Requirement;
use serde_yaml;
use std::path::PathBuf;

pub fn validate_requirements(requirement_dirs: &Vec<PathBuf>) -> Result<()> {
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
    for requirement_file in requirements_files {
        validate_requirement(&requirement_file).context(format!(
            "Failed to validate the requirement: {:?}",
            requirement_file
        ))?;
    }
    Ok(())
}

fn validate_requirement(requirement_path: &PathBuf) -> Result<()> {
    let requirement = std::fs::read_to_string(&requirement_path).context(format!(
        "Failed to read the requirement file: {:?}",
        requirement_path
    ))?;
    serde_yaml::from_str::<Requirement>(&requirement).context(format!(
        "Failed to deserialize the requirement: {:?}",
        requirement_path
    ))?;
    Ok(())
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
        let requirement_str = serde_yaml::to_string(&requirement).unwrap();
        file.write_all(requirement_str.as_bytes()).unwrap();
        validate_requirement(&requirement_path).unwrap();
    }

    #[test]
    fn test_validate_requirement_invalid() {
        let dir = tempdir().unwrap();
        let requirement_path = dir.path().join("requirement.yaml");
        let mut file = File::create(&requirement_path).unwrap();
        let requirement_str = "invalid";
        file.write_all(requirement_str.as_bytes()).unwrap();
        assert!(validate_requirement(&requirement_path).is_err());
    }
}
