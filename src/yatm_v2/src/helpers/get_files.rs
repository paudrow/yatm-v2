use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the files with the given extensions recursively from the directories.
pub fn get_files(dir: &Vec<PathBuf>, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    // Recursively called helper function to process directories
    fn get_files_recursive(
        dir: &PathBuf,
        extensions: &[&str],
        out_files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let paths = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read the directory: {:?}", dir))?;
        for path in paths {
            let entry = path
                .with_context(|| format!("Failed to read the entry in the directory: {:?}", dir))?;
            let file_path = entry.path();
            if file_path.is_dir() {
                // If the path is a directory, recursively search it
                get_files_recursive(&file_path, extensions, out_files)?;
            } else {
                // Otherwise, process the file
                let extension = file_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase());
                if let Some(extension) = extension {
                    if extensions.contains(&extension.as_str()) {
                        out_files.push(file_path);
                    }
                }
            }
        }
        Ok(())
    }

    let mut out_files: Vec<PathBuf> = vec![];
    for dir in dir {
        get_files_recursive(dir, extensions, &mut out_files)?;
    }
    if out_files.is_empty() {
        return Err(anyhow::anyhow!("No files found in directory: {:?}", dir));
    }
    Ok(out_files)
}

#[cfg(test)]
mod test_get_files {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    const YAML_EXTENSIONS: [&str; 1] = ["yaml"];

    #[test]
    fn test_get_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.yaml");
        let mut file = File::create(&file_path).unwrap();
        let file_str = "file";
        file.write_all(file_str.as_bytes()).unwrap();
        let files = get_files(&vec![dir.path().to_path_buf()], &YAML_EXTENSIONS).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);
    }

    #[test]
    fn test_get_files_invalid() {
        let dir = tempdir().unwrap();
        let files = get_files(&vec![dir.path().to_path_buf()], &YAML_EXTENSIONS);
        assert!(files.is_err());
    }

    #[test]
    fn test_get_files_recursive() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("sub_dir");
        std::fs::create_dir(&sub_dir).unwrap();
        let file_path = sub_dir.join("file.yaml");
        let mut file = File::create(&file_path).unwrap();
        let file_str = "file";
        file.write_all(file_str.as_bytes()).unwrap();
        let files = get_files(&vec![dir.path().to_path_buf()], &YAML_EXTENSIONS).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);
    }

    #[test]
    fn test_get_files_recursive_invalid() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("sub_dir");
        std::fs::create_dir(&sub_dir).unwrap();
        let file_path = sub_dir.join("file.txt");
        let mut file = File::create(&file_path).unwrap();
        let file_str = "file";
        file.write_all(file_str.as_bytes()).unwrap();
        let files = get_files(&vec![dir.path().to_path_buf()], &YAML_EXTENSIONS);
        assert!(files.is_err());
    }
}
