use crate::types::LocalIssue;
use octocrab::models::issues::Issue as GithubIssue;

/// The types of matches between a local issue and a github issue
/// Canonical Match is matching permutation and shortname
#[derive(Eq, PartialEq)]
pub enum IssueMatchType {
    Missing,         // No equivalant GitHub Issue found
    Match,           // Matching github issue found
    MatchedWithDiff, // Matching GitHub Issue found but with some differences
}

pub struct GithubIssueMatches {
    pub local_issue: LocalIssue,
    pub github_issue: Option<GithubIssue>,
    pub match_type: IssueMatchType,
}

/// Get local issues that match upstream
pub fn get_local_issues_matches(
    local_issues: &Vec<LocalIssue>,
    github_issues: &Vec<GithubIssue>,
) -> Vec<GithubIssueMatches> {
    let mut results: Vec<GithubIssueMatches> = Vec::<GithubIssueMatches>::new();
    for local_issue in local_issues {
        let gh_issue = github_issues
            .iter()
            .find(|i| is_local_issue_match_github_issue(local_issue, i));

        if gh_issue.is_some() {
            results.push(GithubIssueMatches {
                local_issue: local_issue.clone(),
                github_issue: gh_issue.clone().cloned(),
                match_type: if is_local_issue_identical_github_issue(local_issue, gh_issue.unwrap())
                {
                    IssueMatchType::Match
                } else {
                    IssueMatchType::MatchedWithDiff
                },
            });
        } else {
            results.push(GithubIssueMatches {
                local_issue: local_issue.clone(),
                github_issue: None,
                match_type: IssueMatchType::Missing,
            });
        }
    }
    results
}

// TODO(tfoote) Change this to a specific tag match
fn is_local_issue_match_github_issue(local_issue: &LocalIssue, github_issue: &GithubIssue) -> bool {
    for label in local_issue.labels.iter() {
        let github_issue_label_names = github_issue.labels.iter();
        if !github_issue
            .labels
            .iter()
            .any(|github_label| &github_label.name == label)
        {
            return false;
        }
    }
    return true;
}

fn is_local_issue_identical_github_issue(
    local_issue: &LocalIssue,
    github_issue: &GithubIssue,
) -> bool {
    let title_match: bool = if local_issue.title == github_issue.title {
        true
    } else {
        false
    };

    let body_match: bool =
        if local_issue.text_body == github_issue.body.clone().unwrap_or("".to_string()) {
            true
        } else {
            false
        };

    //TODO (tfoote) check for missing labels

    // Full match
    title_match && body_match
}
