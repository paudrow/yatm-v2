use askama::Template;
use common::types::{Step, TestCase, Action, Expect};
use std::collections::HashMap;

#[derive(Template, Clone)]
#[template(path = "github_issue.md")]
struct GithubIssueTemplate {
    name: String,
    description: String,
    labels: Vec<String>,
    steps: Vec<Step>,
    selected_permutation: HashMap<String, String>,
}

fn main() {
    let test_case = TestCase {
        requirement: common::types::Requirement {
            name: "requirement".to_string(),
            description: "description".to_string(),
            labels: vec!["label".to_string()],
            steps: vec![
                common::types::Step {
                    action: vec![
                        common::types::Action::StdIn(common::types::Terminal {
                            number: 1,
                            text: "text".to_string(),
                        }),
                        common::types::Action::Image("image".to_string()),
                        common::types::Action::Describe("describe".to_string()),
                    ],
                    expect: vec![common::types::Expect::StdOut(common::types::Terminal {
                        number: 1,
                        text: "text".to_string(),
                    })],
                },
                common::types::Step {
                    action: vec![common::types::Action::StdIn(common::types::Terminal {
                        number: 1,
                        text: "text 2".to_string(),
                    })],
                    expect: vec![common::types::Expect::StdOut(common::types::Terminal {
                        number: 1,
                        text: "text 2".to_string(),
                    })],
                },
            ],
        },
        builder_used: common::types::TestCasesBuilder {
            name: "builder".to_string(),
            description: "description".to_string(),
            labels: vec!["label".to_string()],
            set: vec![common::types::SetSteps::Include(common::types::Filter {
                all_labels: Some(vec!["label".to_string()]),
                any_names: Some(vec!["name".to_string()]),
                negate: false,
            })],
            permutations: std::collections::HashMap::new(),
        },
        selected_permutation: std::collections::HashMap::new(),
    };
    // combine the labels from the requirement and the builder
    let mut labels = test_case.builder_used.labels.clone();
    labels.extend(test_case.requirement.labels.clone());


    let template = GithubIssueTemplate {
        name: test_case.requirement.name,
        description: test_case.requirement.description,
        labels,
        steps: test_case.requirement.steps,
        selected_permutation: test_case.selected_permutation,
    };
    println!("{}", template.render().unwrap());
}
