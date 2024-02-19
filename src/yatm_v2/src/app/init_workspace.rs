use crate::types::{Config, RequirementsFile, TestCasesBuilderFile};
use anyhow::{Context, Result};
use serde_yaml;
use std::path::PathBuf;

/// Initialize the configuration directory.
pub fn init_workspace(dir: &PathBuf) -> Result<()> {
    make_sure_empty_dir_exists(dir)?;

    // Create the config file
    let config = Config::default();
    let config_file = dir.join("config.yaml");
    std::fs::write(
        &config_file,
        serde_yaml::to_string(&config).context("Failed to serialize the config")?,
    )
    .context("Failed to write the config file")?;

    // Get the current datetime for naming files
    let datetime_string = chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S").to_string();

    // Create the requirements directory and file
    let requirements_dir = dir.join(&config.new_requirements_dir);
    make_sure_empty_dir_exists(&requirements_dir)?;
    let requirements_file_path =
        requirements_dir.join(format!("requirements-{}.yaml", datetime_string));
    let requirements_file = RequirementsFile::default();
    std::fs::write(
        &requirements_file_path,
        serde_yaml::to_string(&requirements_file)
            .context("Failed to serialize the requirements")?,
    )?;

    // Create the test cases builder directory and file
    let test_cases_builder_dir = dir.join(&config.new_test_cases_builder_dir);
    make_sure_empty_dir_exists(&test_cases_builder_dir)?;
    let test_cases_builder_file =
        test_cases_builder_dir.join(format!("test_cases_builder-{}.yaml", datetime_string));
    std::fs::write(
        &test_cases_builder_file,
        serde_yaml::to_string(&TestCasesBuilderFile::default())
            .context("Failed to serialize the test cases builder")?,
    )
    .context("Failed to write the test cases builder file")?;

    // Create the generated files directory and .gitignore file
    let generated_files_dir = dir.join(&config.generated_files_dir);
    make_sure_empty_dir_exists(&generated_files_dir)?;
    let gitignore_file = dir.join(".gitignore");
    std::fs::write(
        &gitignore_file,
        &format!("/{}", config.generated_files_dir.to_string_lossy()),
    )
    .context("Failed to write the .gitignore file")?;

    Ok(())
}

#[cfg(test)]
mod test_init_config {
    use crate::app::load_config::load_config;

    use super::init_workspace;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn init_config_creates_files() {
        let dir = tempdir().unwrap().path().to_path_buf();
        init_workspace(&dir).unwrap();
        assert!(dir.join("config.yaml").is_file());

        let config = load_config(&dir).unwrap();

        assert!(config.new_requirements_dir.is_dir());
        assert!(config.new_test_cases_builder_dir.is_dir());
    }

    #[test]
    fn init_config_exists_not_empty() {
        let dir = tempdir().unwrap().path().to_path_buf();
        fs::create_dir(&dir).unwrap();
        fs::File::create(dir.join("file")).unwrap();
        assert!(init_workspace(&dir).is_err());
    }

    #[test]
    fn init_config_exists_file() {
        let dir = tempdir().unwrap().path().to_path_buf();
        fs::File::create(&dir).unwrap();
        assert!(init_workspace(&dir).is_err());
    }
}

fn make_sure_empty_dir_exists(dir: &PathBuf) -> Result<()> {
    if dir.is_file() {
        anyhow::bail!(format!("Path already exists and is a file: {:?}", dir));
    }
    if dir.is_dir() {
        if !is_empty_directory(dir)? {
            anyhow::bail!(format!("Directory is not empty: {:?}", dir));
        }
    } else {
        std::fs::create_dir(dir).context("Failed to create the directory")?;
    }
    Ok(())
}

#[cfg(test)]
mod test_make_sure_empty_dir_exists {
    use super::is_empty_directory;
    use super::make_sure_empty_dir_exists;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn create_empty_dir() {
        let dir = tempdir().unwrap().path().to_path_buf();
        make_sure_empty_dir_exists(&dir).unwrap();
        assert!(dir.is_dir());
        assert!(is_empty_directory(&dir).unwrap());
    }

    #[test]
    fn create_empty_dir_exists() {
        let dir = tempdir().unwrap().path().to_path_buf();
        fs::create_dir(&dir).unwrap();
        make_sure_empty_dir_exists(&dir).unwrap();
        assert!(dir.is_dir());
        assert!(is_empty_directory(&dir).unwrap());
    }

    #[test]
    fn create_empty_dir_exists_not_empty() {
        let dir = tempdir().unwrap().path().to_path_buf();
        fs::create_dir(&dir).unwrap();
        fs::File::create(dir.join("file")).unwrap();
        assert!(make_sure_empty_dir_exists(&dir).is_err());
    }

    #[test]
    fn create_empty_dir_exists_file() {
        let dir = tempdir().unwrap().path().to_path_buf();
        fs::File::create(&dir).unwrap();
        assert!(make_sure_empty_dir_exists(&dir).is_err());
    }
}

fn is_empty_directory(dir: &PathBuf) -> Result<bool> {
    if dir.is_dir() {
        let mut entries = std::fs::read_dir(dir).context("Failed to read the directory")?;
        if entries.next().is_none() {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod test_is_empty_directory {
    use super::is_empty_directory;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn is_empty() {
        let dir = tempdir().unwrap().path().to_path_buf();
        fs::create_dir(&dir).unwrap();
        assert!(is_empty_directory(&dir).unwrap());
    }

    #[test]
    fn is_not_empty() {
        let dir = tempdir().unwrap().path().to_path_buf();
        fs::create_dir(&dir).unwrap();
        fs::File::create(dir.join("file")).unwrap();
        assert!(!is_empty_directory(&dir).unwrap());
    }

    #[test]
    fn is_not_exists() {
        let dir = tempdir().unwrap().path().to_path_buf();
        assert!(!is_empty_directory(&dir).unwrap());
    }
}
