use crate::types::LocalIssue;
use octocrab::models::issues::Issue as GithubIssue;

/// The types of matches between a local issue and a github issue
/// Canonical Match is matching permutation and shortname
enum IssueMatchType {
    Missing,         // No equivalant GitHub Issue found
    Match,           // Matching github issue found
    MatchedWithDiff, // Matching GitHub Issue found but with some differences
}

/// Potential differences
/// Different Title
/// Different Body
/// Extra Lables
/// Missing Labels

struct GithubIssueHelper {
    pub title: String,
    pub labels: Vec<String>,
}

// struct GithubIssueMatches {
//     pub local_issue: LocalIssue,
//     pub github_issue: Option<GithubIssue>,
//     pub match_type: IssueMatchType,
// }

// /// Get local issues that match upstream
// pub fn get_local_issues_matches(
//     local_issues: &Vec<LocalIssue>,
//     github_issues: &Vec<GithubIssue>,
// ) -> Vec<GithubIssueMatches> {
//     let mut results: Vec<GithubIssueMatches> = Vec::<GithubIssueMatches>::new();
//     for local_issue in local_issues {
//         let gh_issue = github_issues.iter().find(|i| is_local_issue_match_github_issue(local_issue, i));

//         if gh_issue.is_some() {
//             results.push(GithubIssueMatches{
//                 local_issue: local_issue.clone(),
//                 github_issue: gh_issue.clone().cloned(),
//                 match_type: if is_local_issue_identical_github_issue(local_issue, gh_issue.unwrap()) { IssueMatchType::Match} else { IssueMatchType::MatchedWithDiff},
//             });
//         }
//         else {
//             results.push(GithubIssueMatches{
//                 local_issue: local_issue.clone(),
//                 github_issue: None,
//                 match_type: IssueMatchType::Missing,
//             });
//         }
//     }
//     results
// }

// fn is_local_issue_match_github_issue(
//     local_issue: &LocalIssue,
//     github_issue: &GithubIssue,
// ) -> bool {
//     for label in local_issue.labels.iter() {
//         if !github_issue
//             .labels
//             .iter()
//             .any(|github_label| &github_label.name.clone().to_string() == label)
//         {
//             return true;
//         }
//     }
//     false
// }

// fn is_local_issue_identical_github_issue(
//     local_issue: &LocalIssue,
//     github_issue: &GithubIssue,
// ) -> bool {

//     let title_match: bool =
//         if local_issue.title == github_issue.title { true } else { false };

//     let body_match: bool =
//         if local_issue.text_body == github_issue.body_text.clone().unwrap() { true } else { false };

//     // Full match
//     title_match && body_match

// }

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
