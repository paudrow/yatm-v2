use crate::types::LocalIssue;
use anyhow::{Context, Result};
use askama::Template;
use common::types::{Action, Expect, Link, Step, TestCase};
use std::collections::HashMap;

#[derive(Template, Clone)]
#[template(path = "github_issue.md")]
struct GithubIssueTemplate {
    description: String,
    steps: Vec<Step>,
    links: Vec<Link>,
    selected_permutation: HashMap<String, String>,
    minimum_permutations_to_render: usize,
}

pub fn test_case_to_markdown(
    test_case: TestCase,
    workspace_version: &String,
) -> Result<LocalIssue> {
    let labels = get_labels(&test_case, workspace_version);

    let template = GithubIssueTemplate {
        description: test_case.requirement.description,
        steps: test_case.requirement.steps,
        links: test_case.requirement.links.unwrap_or_default(),
        selected_permutation: test_case.selected_permutation,
        minimum_permutations_to_render: test_case.builder_used.minimum_permutations_to_render
            as usize,
    };
    let text_body = template.render().context("Failed to render the template")?;

    Ok(LocalIssue {
        labels,
        title: test_case.requirement.name,
        text_body,
    })
}

fn get_labels(test_case: &TestCase, workspace_version: &String) -> Vec<String> {
    let mut labels: Vec<String> = vec![];
    if let Some(labels_) = test_case.builder_used.labels.clone() {
        labels.extend(labels_);
    }
    if let Some(labels_) = test_case.requirement.labels.clone() {
        labels.extend(labels_);
    }
    labels.push(format!("requirement: {}", test_case.requirement.shortname.clone().unwrap_or(test_case.requirement.name.clone())));
    labels.extend(permutation_to_labels(&test_case.selected_permutation));
    labels.push(project_version_to_label(workspace_version));
    labels
}

pub fn permutation_to_labels(permutations: &HashMap<String, String>) -> Vec<String> {
    let mut labels: Vec<String> = vec![];
    for (key, value) in permutations.iter() {
        labels.push(format!("{}: {}", key, value));
    }
    labels.sort();
    labels
}

pub fn project_version_to_label(workspace_version: &String) -> String {
    format!("version: {}", workspace_version)
}
