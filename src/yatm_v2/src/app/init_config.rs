use crate::types::Config;
use anyhow::{Context, Result};
use serde_yaml;
use std::path::PathBuf;

pub fn init_config(dir: &PathBuf) -> Result<()> {
    make_sure_empty_dir_exists(dir)?;

    let config = Config::default();
    let config_file = dir.join("config.yaml");
    std::fs::write(
        &config_file,
        serde_yaml::to_string(&config).context("Failed to serialize the config")?,
    )
    .context("Failed to write the config file")?;

    if config.requirements_dirs.len() == 1 {
        let requirements_dir = dir.join(&config.requirements_dirs[0]);
        make_sure_empty_dir_exists(&requirements_dir)?;
    }

    let generated_files_dir = dir.join(&config.generated_files_dir);
    make_sure_empty_dir_exists(&generated_files_dir)?;

    // add a .gitignore file with the generated_files_dir
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
    use super::init_config;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn init_config_creates_files() {
        let dir = tempdir().unwrap().path().to_path_buf();
        init_config(&dir).unwrap();
        assert!(dir.join("config.yaml").is_file());
        assert!(dir.join("requirements").is_dir());
        assert!(dir.join(".generated_files").is_dir());
        assert!(dir.join(".gitignore").is_file());
    }

    #[test]
    fn init_config_exists_not_empty() {
        let dir = tempdir().unwrap().path().to_path_buf();
        fs::create_dir(&dir).unwrap();
        fs::File::create(dir.join("file")).unwrap();
        assert!(init_config(&dir).is_err());
    }

    #[test]
    fn init_config_exists_file() {
        let dir = tempdir().unwrap().path().to_path_buf();
        fs::File::create(&dir).unwrap();
        assert!(init_config(&dir).is_err());
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
