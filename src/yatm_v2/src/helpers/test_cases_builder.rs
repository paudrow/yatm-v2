use crate::constants::YAML_EXTENSIONS;
use crate::helpers::get_files;
use anyhow::{Context, Result};
use common::types::TestCasesBuilder;
use common::types::TestCasesBuilderFile;
use serde_yaml;
use std::path::PathBuf;

/// Validate the test cases builder file.
pub fn validate_test_cases_builder_file(test_cases_builder_path: &PathBuf) -> Result<()> {
    get_test_cases_builders_from_file(test_cases_builder_path)?;
    Ok(())
}

#[cfg(test)]
mod test_validate_test_cases_builder {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_validate_test_cases_builder() {
        let dir = tempdir().unwrap();
        let test_cases_builder_path = dir.path().join("test_cases_builder.yaml");
        let mut file = File::create(&test_cases_builder_path).unwrap();
        let test_cases_builder_file = TestCasesBuilderFile::default();
        let test_cases_builder = serde_yaml::to_string(&test_cases_builder_file).unwrap();
        file.write_all(test_cases_builder.as_bytes()).unwrap();
        validate_test_cases_builder_file(&test_cases_builder_path).unwrap();
    }

    #[test]
    fn test_validate_test_cases_builder_invalid() {
        let dir = tempdir().unwrap();
        let test_cases_builder_path = dir.path().join("test_cases_builder.yaml");
        let mut file = File::create(&test_cases_builder_path).unwrap();
        let test_cases_builder_str = "invalid";
        file.write_all(test_cases_builder_str.as_bytes()).unwrap();
        assert!(validate_test_cases_builder_file(&test_cases_builder_path).is_err());
    }
}

/// Get the test cases builders from the files.
pub fn get_test_cases_builders_from_files(
    test_cases_builder_dirs: &Vec<PathBuf>,
) -> Result<Vec<TestCasesBuilder>> {
    let mut all_test_cases_builders: Vec<TestCasesBuilder> = vec![];
    for test_cases_builder_dir in test_cases_builder_dirs {
        let files = get_files(&vec![test_cases_builder_dir.clone()], &YAML_EXTENSIONS)?;
        for file in files {
            let test_cases_builders = get_test_cases_builders_from_file(&file)?;
            all_test_cases_builders.extend(test_cases_builders);
        }
    }
    Ok(all_test_cases_builders)
}

/// Get the test cases builders from the file.
pub fn get_test_cases_builders_from_file(
    test_cases_builder_path: &PathBuf,
) -> Result<Vec<TestCasesBuilder>> {
    let test_cases_builder_file =
        std::fs::read_to_string(&test_cases_builder_path).context(format!(
            "No test cases builder file found: {:?}",
            test_cases_builder_path
        ))?;
    let test_cases_builder_file =
        serde_yaml::from_str::<TestCasesBuilderFile>(&test_cases_builder_file).context(format!(
            "Failed to deserialize the test cases builder: {:?}",
            test_cases_builder_path
        ))?;
    Ok(test_cases_builder_file.test_cases_builders)
}
