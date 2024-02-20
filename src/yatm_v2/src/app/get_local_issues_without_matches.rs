use crate::types::LocalIssue;
use octocrab::models::issues::Issue as GithubIssue;

struct GithubIssueHelper {
    pub title: String,
    pub labels: Vec<String>,
}

/// Get local issues that do not have a match in the github issues
///
/// Note that github issues can have more labels than the local issues and
/// still be considered a match. This is to allow for additional tags to be added
/// to the github issues, such as help wanted, bug, etc.
pub fn get_local_issues_without_matches(
    local_issues: &Vec<LocalIssue>,
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
    get_local_issues_without_matches_helper(&local_issues, &github_issues)
}

fn get_local_issues_without_matches_helper(
    local_issues: &Vec<LocalIssue>,
    github_issues: &Vec<GithubIssueHelper>,
) -> Vec<LocalIssue> {
    local_issues
        .iter()
        .filter(|local_issue| {
            !github_issues.iter().any(|github_issue| {
                is_local_issue_match_github_issue_helper(local_issue, github_issue)
            })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod test_get_local_issues_without_matches {
    use super::get_local_issues_without_matches_helper;
    use super::{GithubIssueHelper, LocalIssue};

    #[test]
    fn matches() {
        let local_issues = vec![
            LocalIssue {
                labels: vec!["label1".to_string()],
                title: "title".to_string(),
                text_body: "text_body".to_string(),
            },
            LocalIssue {
                labels: vec!["label2".to_string()],
                title: "title2".to_string(),
                text_body: "text_body2".to_string(),
            },
        ];
        let github_issues: Vec<_> = vec![
            GithubIssueHelper {
                labels: vec!["label3".to_string()],
                title: "title3".to_string(),
            },
            GithubIssueHelper {
                labels: vec!["label4".to_string()],
                title: "title4".to_string(),
            },
        ];
        let result = get_local_issues_without_matches_helper(&local_issues, &github_issues);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn no_matches() {
        let local_issues = vec![
            LocalIssue {
                labels: vec!["label1".to_string()],
                title: "title".to_string(),
                text_body: "text_body".to_string(),
            },
            LocalIssue {
                labels: vec!["label2".to_string()],
                title: "title2".to_string(),
                text_body: "text_body2".to_string(),
            },
        ];
        let github_issues: Vec<_> = vec![
            GithubIssueHelper {
                labels: vec!["label1".to_string()],
                title: "title".to_string(),
            },
            GithubIssueHelper {
                labels: vec!["label2".to_string()],
                title: "title2".to_string(),
            },
        ];
        let result = get_local_issues_without_matches_helper(&local_issues, &github_issues);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn one_match() {
        let local_issues = vec![
            LocalIssue {
                labels: vec!["label1".to_string()],
                title: "title".to_string(),
                text_body: "text_body".to_string(),
            },
            LocalIssue {
                labels: vec!["label2".to_string()],
                title: "title2".to_string(),
                text_body: "text_body2".to_string(),
            },
        ];
        let github_issues: Vec<_> = vec![
            GithubIssueHelper {
                labels: vec!["label1".to_string()],
                title: "title".to_string(),
            },
            GithubIssueHelper {
                labels: vec!["label3".to_string()],
                title: "title3".to_string(),
            },
        ];
        let result = get_local_issues_without_matches_helper(&local_issues, &github_issues);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "title2");
    }
}

fn is_local_issue_match_github_issue_helper(
    local_issue: &LocalIssue,
    github_issue: &GithubIssueHelper,
) -> bool {
    if local_issue.title != github_issue.title {
        return false;
    }
    if local_issue.labels.len() > github_issue.labels.len() {
        return false;
    }
    for label in local_issue.labels.iter() {
        if !github_issue
            .labels
            .iter()
            .any(|github_label| github_label == label)
        {
            return false;
        }
    }
    return true;
}

#[cfg(test)]
mod test_is_local_issue_match_github_issue {
    use super::is_local_issue_match_github_issue_helper;
    use super::{GithubIssueHelper, LocalIssue};

    #[test]
    fn is_match() {
        let local_issue = LocalIssue {
            labels: vec!["label1".to_string()],
            title: "title".to_string(),
            text_body: "text_body".to_string(),
        };
        let github_issue = GithubIssueHelper {
            labels: vec!["label1".to_string()],
            title: "title".to_string(),
        };
        assert!(is_local_issue_match_github_issue_helper(
            &local_issue,
            &github_issue
        ));
    }

    #[test]
    fn is_match_with_github_issue_having_more_labels() {
        let local_issue = LocalIssue {
            labels: vec!["label1".to_string()],
            title: "title".to_string(),
            text_body: "text_body".to_string(),
        };
        let github_issue = GithubIssueHelper {
            labels: vec!["label1".to_string(), "label2".to_string()],
            title: "title".to_string(),
        };
        assert!(is_local_issue_match_github_issue_helper(
            &local_issue,
            &github_issue
        ));
    }

    #[test]
    fn is_not_match_label() {
        let local_issue = LocalIssue {
            labels: vec!["label1".to_string()],
            title: "title".to_string(),
            text_body: "text_body".to_string(),
        };
        let github_issue = GithubIssueHelper {
            labels: vec!["label2".to_string()],
            title: "title".to_string(),
        };
        assert!(!is_local_issue_match_github_issue_helper(
            &local_issue,
            &github_issue
        ));
    }

    #[test]
    fn is_not_match_title() {
        let local_issue = LocalIssue {
            labels: vec!["label1".to_string()],
            title: "title".to_string(),
            text_body: "text_body".to_string(),
        };
        let github_issue = GithubIssueHelper {
            labels: vec!["label1".to_string()],
            title: "title2".to_string(),
        };
        assert!(!is_local_issue_match_github_issue_helper(
            &local_issue,
            &github_issue
        ));
    }

    #[test]
    fn is_not_match_label_and_title() {
        let local_issue = LocalIssue {
            labels: vec!["label1".to_string()],
            title: "title".to_string(),
            text_body: "text_body".to_string(),
        };
        let github_issue = GithubIssueHelper {
            labels: vec!["label2".to_string()],
            title: "title2".to_string(),
        };
        assert!(!is_local_issue_match_github_issue_helper(
            &local_issue,
            &github_issue
        ));
    }

    #[test]
    fn is_not_match_missing_label() {
        let local_issue = LocalIssue {
            labels: vec!["label1".to_string(), "label2".to_string()],
            title: "title".to_string(),
            text_body: "text_body".to_string(),
        };
        let github_issue = GithubIssueHelper {
            labels: vec!["label2".to_string()],
            title: "title".to_string(),
        };
        assert!(!is_local_issue_match_github_issue_helper(
            &local_issue,
            &github_issue
        ));
    }
}
