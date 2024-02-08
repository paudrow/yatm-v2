use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::utils::template::LocalIssue;
use octocrab::models::issues::Issue as GithubIssue;
use url::Url;

struct GithubIssueHelper {
    pub title: String,
    pub labels: Vec<String>,
}

pub fn get_local_issues_without_matches(
    local_issues: Vec<LocalIssue>,
    github_issues: &Vec<GithubIssue>,
) -> Vec<LocalIssue> {
    let github_issues: Vec<GithubIssueHelper> = github_issues
        .iter()
        .map(|issue| GithubIssueHelper {
            title: issue.title.clone(),
            labels: issue
                .labels
                .clone()
                .iter()
                .map(|label| label.name.clone())
                .collect(),
        })
        .collect();
    get_local_issues_without_matches_helper(local_issues, &github_issues)
}

fn get_local_issues_without_matches_helper(
    local_issues: Vec<LocalIssue>,
    github_issues: &Vec<GithubIssueHelper>,
) -> Vec<LocalIssue> {
    let github_issue_hashes: Vec<_> = github_issues
        .iter()
        .map(|issue| get_issue_hash(issue.title.clone(), issue.labels.clone()))
        .collect();
    local_issues
        .iter()
        .filter(|local_issue| {
            let local_labels = local_issue.labels.clone();
            let local_hash = get_issue_hash(local_issue.title.clone(), local_labels);
            !github_issue_hashes.contains(&local_hash)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod test_get_local_issues_without_matches {
    use super::get_local_issues_without_matches_helper;
    use crate::utils::{github_utils::GithubIssueHelper, template::LocalIssue};

    #[test]
    fn no_matches() {
        let local_issues = vec![
            LocalIssue {
                labels: vec!["label1".to_string()],
                title: "title".to_string(),
                text_body: "text".to_string(),
            },
            LocalIssue {
                labels: vec!["label2".to_string()],
                title: "title2".to_string(),
                text_body: "text2".to_string(),
            },
        ];
        let github_issues = vec![GithubIssueHelper {
            labels: vec![],
            title: "title3".to_string(),
        }];
        let result = get_local_issues_without_matches_helper(local_issues, &github_issues);
        assert_eq!(result.len(), 2, "All local issues should be returned");
    }

    #[test]
    fn one_match() {
        let local_issues = vec![
            LocalIssue {
                labels: vec!["label1".to_string(), "label2".to_string()],
                title: "title".to_string(),
                text_body: "text".to_string(),
            },
            LocalIssue {
                labels: vec!["label2".to_string()],
                title: "title2".to_string(),
                text_body: "text2".to_string(),
            },
        ];
        let github_issues = vec![GithubIssueHelper {
            labels: vec!["label1".to_string(), "label2".to_string()],
            title: "title".to_string(),
        }];
        let result = get_local_issues_without_matches_helper(local_issues, &github_issues);
        assert_eq!(result.len(), 1, "One local issue should be returned");
        assert_eq!(
            result[0].title, "title2",
            "The second local issue should be returned"
        );
    }

    #[test]
    fn all_matches() {
        let local_issues = vec![
            LocalIssue {
                labels: vec!["label1".to_string(), "label2".to_string()],
                title: "title".to_string(),
                text_body: "text".to_string(),
            },
            LocalIssue {
                labels: vec!["label2".to_string()],
                title: "title2".to_string(),
                text_body: "text2".to_string(),
            },
        ];
        let github_issues = vec![
            GithubIssueHelper {
                labels: vec!["label1".to_string(), "label2".to_string()],
                title: "title".to_string(),
            },
            GithubIssueHelper {
                labels: vec!["label2".to_string()],
                title: "title2".to_string(),
            },
        ];
        let result = get_local_issues_without_matches_helper(local_issues, &github_issues);
        assert_eq!(result.len(), 0, "No local issues should be returned");
    }
}

pub fn get_issue_hash(title: String, labels: Vec<String>) -> u64 {
    let mut hasher = DefaultHasher::new();
    title.hash(&mut hasher);

    let mut labels = labels.clone();
    labels.sort();
    for label in labels {
        label.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod test_get_issue_hash {
    use super::get_issue_hash;

    #[test]
    fn different_order_labels_should_still_match() {
        let hash1 = get_issue_hash(
            "title".to_string(),
            vec!["label1".to_string(), "label2".to_string()],
        );
        let hash2 = get_issue_hash(
            "title".to_string(),
            vec!["label2".to_string(), "label1".to_string()],
        );
        assert_eq!(hash1, hash2, "Hashes should be equal");
    }

    #[test]
    fn different_shouldnt_match() {
        let hash1 = get_issue_hash(
            "title".to_string(),
            vec!["label1".to_string(), "label2".to_string()],
        );
        let hash2 = get_issue_hash(
            "title".to_string(),
            vec!["label2".to_string(), "label3".to_string()],
        );
        let hash3 = get_issue_hash(
            "title - new".to_string(),
            vec!["label1".to_string(), "label2".to_string()],
        );
        let hash4 = get_issue_hash(
            "TITLE".to_string(),
            vec!["label1".to_string(), "label2".to_string()],
        );

        assert_ne!(hash1, hash2, "Labels don't match");
        assert_ne!(hash1, hash3, "Title text doesn't match");
        assert_ne!(hash1, hash4, "Title casing doesn't match");
    }
}
