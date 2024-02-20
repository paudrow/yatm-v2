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
}

pub fn test_case_to_markdown(test_case: TestCase) -> Result<LocalIssue> {
    let labels = get_labels(&test_case);

    let template = GithubIssueTemplate {
        description: test_case.requirement.description,
        steps: test_case.requirement.steps,
        links: test_case.requirement.links.unwrap_or_default(),
        selected_permutation: test_case.selected_permutation,
    };
    let text_body = template.render().context("Failed to render the template")?;

    Ok(LocalIssue {
        labels,
        title: test_case.requirement.name,
        text_body,
    })
}

fn get_labels(test_case: &TestCase) -> Vec<String> {
    let mut labels: Vec<String> = vec![];
    if let Some(labels_) = test_case.builder_used.labels.clone() {
        labels.extend(labels_);
    }
    if let Some(labels_) = test_case.requirement.labels.clone() {
        labels.extend(labels_);
    }
    for (key, value) in test_case.selected_permutation.iter() {
        labels.push(format!("{}: {}", key, value));
    }
    labels.push(format!("version: {}", test_case.builder_used.version));
    labels
}