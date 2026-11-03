use crate::constants::YAML_EXTENSIONS;
use crate::helpers::get_files;
use anyhow::{Context, Result};
use common::types::{Requirement, RequirementSource, RequirementsFile};
use serde_yaml;
use std::path::{Path, PathBuf};
use std::process::Command;

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

    #[test]
    fn test_get_web_browsing_link() {
        let git_url = "https://github.com/ros2/demos";
        let branch = "iron";
        let path = "README.md";
        let link_github = get_web_browsing_link(git_url, branch, path, Some(12));
        assert_eq!(
            link_github,
            Some("https://github.com/ros2/demos/blob/iron/README.md#L12".to_string())
        );

        let gitlab_url = "https://gitlab.com/ros2/demos";
        let link_gitlab = get_web_browsing_link(gitlab_url, branch, path, None);
        assert_eq!(
            link_gitlab,
            Some("https://gitlab.com/ros2/demos/-/blob/iron/README.md".to_string())
        );

        let unrecognized = "https://other-git.com/ros2/demos";
        let link_unrecognized = get_web_browsing_link(unrecognized, branch, path, Some(5));
        assert_eq!(link_unrecognized, None);
    }

    #[test]
    fn test_select_source_branch_priority() {
        let s = |v: &str| Some(v.to_string());
        assert_eq!(select_source_branch(s("rel"), s("base"), s("dflt")), "rel");
        assert_eq!(select_source_branch(s("  "), s("base"), s("dflt")), "base");
        assert_eq!(select_source_branch(None, s(""), s("dflt")), "dflt");
        assert_eq!(select_source_branch(None, None, None), "main");
    }

    #[test]
    fn test_origin_default_branch_ignores_checked_out_branch() {
        let dir = tempdir().unwrap();
        let run = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@example.com")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@example.com")
                .output()
                .unwrap();
            assert!(
                status.status.success(),
                "git {:?} failed: {:?}",
                args,
                status
            );
        };
        run(&["init", "-q", "-b", "feature_branch"]);
        run(&["commit", "-q", "--allow-empty", "-m", "init"]);
        run(&["update-ref", "refs/remotes/origin/develop", "HEAD"]);
        run(&[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/develop",
        ]);
        assert_eq!(
            get_origin_default_branch(dir.path()),
            Some("develop".to_string())
        );

        let no_remote = tempdir().unwrap();
        Command::new("git")
            .args(&["init", "-q"])
            .current_dir(no_remote.path())
            .output()
            .unwrap();
        assert_eq!(get_origin_default_branch(no_remote.path()), None);
    }

    #[test]
    fn test_find_line_number() {
        let content = "requirements:\n  - name: First Requirement\n    labels:\n      - test\n  - name: Second Requirement";
        assert_eq!(find_line_number(content, "First Requirement"), Some(2));
        assert_eq!(find_line_number(content, "Second Requirement"), Some(5));
        assert_eq!(find_line_number(content, "Third Requirement"), None);
    }

    #[test]
    fn test_clean_repository_url() {
        assert_eq!(
            clean_repository_url("https://github.com/owner/repo.git"),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            clean_repository_url("git@github.com:owner/repo.git"),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            clean_repository_url("git@gitlab.com:owner/repo.git"),
            "https://gitlab.com/owner/repo"
        );
    }

    #[test]
    fn test_get_requirements_from_multiple_files() {
        let dir = tempdir().unwrap();
        let requirement_path1 = dir.path().join("requirement1.yaml");
        let mut file1 = File::create(&requirement_path1).unwrap();
        let mut requirements_file1 = RequirementsFile::default();
        requirements_file1.requirements[0].name = "req1".to_string();
        let req_str1 = serde_yaml::to_string(&requirements_file1).unwrap();
        file1.write_all(req_str1.as_bytes()).unwrap();

        let requirement_path2 = dir.path().join("requirement2.yaml");
        let mut file2 = File::create(&requirement_path2).unwrap();
        let mut requirements_file2 = RequirementsFile::default();
        requirements_file2.requirements[0].name = "req2".to_string();
        let req_str2 = serde_yaml::to_string(&requirements_file2).unwrap();
        file2.write_all(req_str2.as_bytes()).unwrap();

        let reqs = get_requirements_from_files(&vec![dir.path().to_path_buf()]).unwrap();
        assert_eq!(reqs.len(), 2);
        let mut names: Vec<String> = reqs.into_iter().map(|r| r.name).collect();
        names.sort();
        assert_eq!(names, vec!["req1".to_string(), "req2".to_string()]);
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
    let requirement_str = std::fs::read_to_string(&requirement_path).context(format!(
        "Failed to read the requirement file: {:?}",
        requirement_path
    ))?;
    let mut requirements_file = serde_yaml::from_str::<RequirementsFile>(&requirement_str)
        .context(format!(
            "Failed to deserialize the requirement: {:?}",
            requirement_path
        ))?;

    let git_info = get_git_info(requirement_path);
    let relative_path = get_relative_path(requirement_path);

    for req in &mut requirements_file.requirements {
        let line_num = find_line_number(&requirement_str, &req.name);
        if let Some((repo, branch)) = &git_info {
            if let Some(web_link) = get_web_browsing_link(repo, branch, &relative_path, line_num) {
                req.source = Some(RequirementSource {
                    repo: repo.clone(),
                    branch: branch.clone(),
                    filepath: relative_path.clone(),
                    line: line_num,
                    web_browsing_link: web_link,
                });
            }
        }
    }

    Ok(requirements_file.requirements)
}

pub fn get_web_browsing_link(
    repo_url: &str,
    branch: &str,
    filepath: &str,
    line: Option<usize>,
) -> Option<String> {
    let line_suffix = match line {
        Some(l) => format!("#L{}", l),
        None => "".to_string(),
    };
    if repo_url.contains("github.com") {
        Some(format!(
            "{}/blob/{}/{}{}",
            repo_url, branch, filepath, line_suffix
        ))
    } else if repo_url.contains("gitlab.com") {
        Some(format!(
            "{}/-/blob/{}/{}{}",
            repo_url, branch, filepath, line_suffix
        ))
    } else {
        None
    }
}

/// Environment variable overriding the branch used for requirement source links.
pub const SOURCE_BRANCH_ENV: &str = "YATM_SOURCE_BRANCH";

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Choose the branch to link requirement sources to, in priority order:
/// 1. explicit override (`--source-branch` / `YATM_SOURCE_BRANCH`)
/// 2. `GITHUB_BASE_REF` (the target branch of a GitHub Actions pull request)
/// 3. the remote's default branch (`origin/HEAD`)
/// 4. `main`
///
/// The currently checked-out branch is intentionally *not* used, so that links
/// point at the branch the requirements will land on rather than a feature branch.
fn select_source_branch(
    override_branch: Option<String>,
    github_base_ref: Option<String>,
    origin_default: Option<String>,
) -> String {
    non_empty(override_branch)
        .or_else(|| non_empty(github_base_ref))
        .or_else(|| non_empty(origin_default))
        .unwrap_or_else(|| "main".to_string())
}

/// Read the remote's default branch from the local `refs/remotes/origin/HEAD`
/// (set by `git clone` or `git remote set-head origin --auto`), without network access.
fn get_origin_default_branch(dir: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(&[
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ])
        .current_dir(dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let short = String::from_utf8_lossy(&output.stdout).trim().to_string();
    non_empty(Some(
        short.strip_prefix("origin/").unwrap_or(&short).to_string(),
    ))
}

pub fn get_git_info(path: &Path) -> Option<(String, String)> {
    let dir = path.parent().unwrap_or(path);
    let repo = Command::new("git")
        .args(&["config", "--get", "remote.origin.url"])
        .current_dir(dir)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Some(clean_repository_url(&url))
            } else {
                None
            }
        })?;

    let branch = select_source_branch(
        std::env::var(SOURCE_BRANCH_ENV).ok(),
        std::env::var("GITHUB_BASE_REF").ok(),
        get_origin_default_branch(dir),
    );

    Some((repo, branch))
}

fn clean_repository_url(url: &str) -> String {
    let mut url = url.trim();
    if url.ends_with(".git") {
        url = &url[..url.len() - 4];
    }
    if url.starts_with("git@github.com:") {
        format!("https://github.com/{}", &url[15..])
    } else if url.starts_with("git@gitlab.com:") {
        format!("https://gitlab.com/{}", &url[15..])
    } else {
        url.to_string()
    }
}

pub fn get_relative_path(path: &Path) -> String {
    let dir = path.parent().unwrap_or(path);
    let toplevel = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(PathBuf::from(
                    String::from_utf8_lossy(&output.stdout).trim().to_string(),
                ))
            } else {
                None
            }
        });

    if let Some(top) = toplevel {
        if let Ok(rel) = path.strip_prefix(&top) {
            return rel.to_string_lossy().to_string();
        }
    }
    path.to_string_lossy().to_string()
}

pub fn find_line_number(file_content: &str, req_name: &str) -> Option<usize> {
    for (i, line) in file_content.lines().enumerate() {
        if line.contains("name:") && line.contains(req_name) {
            return Some(i + 1);
        }
    }
    None
}
