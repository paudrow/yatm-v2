mod app;
mod github;
mod test_cases;
mod types;
mod utils;

use anyhow::{Context, Ok, Result};
use types::LocalIssue;

use crate::app::cli;
use crate::github::get_local_issues_without_matches;
use crate::github::Github;
use crate::test_cases::test_case_to_markdown;

use chrono;
use common::types::TestCase;
use dotenv::dotenv;
use octocrab::Octocrab;
use std::collections::HashMap;
use tokio;

fn demo_template() {
    let mut selected_permutation: HashMap<String, String> = HashMap::new();
    selected_permutation.insert("key".to_string(), "value".to_string());
    selected_permutation.insert("key2".to_string(), "value2".to_string());

    let test_case = TestCase {
        requirement: common::types::Requirement {
            name: "requirement".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["req-label".to_string()]),
            // links: None,
            links: Some(vec![
                common::types::Link {
                    name: "link".to_string(),
                    url: "www.url.com".to_string(),
                },
                common::types::Link {
                    name: "link2".to_string(),
                    url: "www.url2.com".to_string(),
                },
            ]),
            steps: vec![
                common::types::Step {
                    action: vec![
                        common::types::Action::StdIn(common::types::Terminal {
                            number: 1,
                            text: "text".to_string(),
                        }),
                        common::types::Action::Image("image".to_string()),
                        common::types::Action::Describe("describe".to_string()),
                        common::types::Action::Url(common::types::Link {
                            name: "link".to_string(),
                            url: "www.url.com".to_string(),
                        }),
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
                    expect: vec![
                        common::types::Expect::StdOut(common::types::Terminal {
                            number: 1,
                            text: "text 2".to_string(),
                        }),
                        common::types::Expect::StdErr(common::types::Terminal {
                            number: 1,
                            text: "text 3".to_string(),
                        }),
                        common::types::Expect::Image("image".to_string()),
                        common::types::Expect::Describe("describe".to_string()),
                        common::types::Expect::Url(common::types::Link {
                            name: "link".to_string(),
                            url: "www.url.com".to_string(),
                        }),
                    ],
                },
            ],
        },
        builder_used: common::types::TestCasesBuilder {
            name: "builder".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["builder-label".to_string()]),
            set: vec![common::types::SetSteps::Include(common::types::Filter {
                all_labels: Some(vec!["label".to_string()]),
                any_names: Some(vec!["name".to_string()]),
                negate: false,
            })],
            permutations: std::collections::HashMap::new(),
            version: 1,
        },
        // selected_permutation: std::collections::HashMap::new(),
        selected_permutation,
    };

    let result = test_case_to_markdown(test_case).unwrap();
    println!("{}", result.title);
    println!("{}", result.text_body);
    println!("{:?}", result.labels);
}

#[tokio::main]
async fn main() -> Result<()> {
    // let gh = Github::new("paudrow".to_string(), "test-yatm-v2".to_string())?;

    // // for _ in 0..300 {
    // //     let issue_title = format!(
    // //         "My issue {}",
    // //         chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    // //     );
    // //     gh.create_issue(
    // //         issue_title.clone(),
    // //         "My issue body".to_string(),
    // //         vec![String::from("label")],
    // //     )
    // //     .await?;
    // // }
    // let local_issues = vec![
    //     LocalIssue {
    //         labels: vec!["label".to_string()],
    //         title: "My issue 2021-08-25 14:00:00".to_string(),
    //         text_body: "My issue body".to_string(),
    //     },
    //     LocalIssue {
    //         labels: vec!["label".to_string()],
    //         title: "My issue 2021-08-25 14:00:01".to_string(),
    //         text_body: "My issue body".to_string(),
    //     },
    // ];
    // let github_issues = gh.get_issues(Some(octocrab::params::State::Open)).await?;
    // let local_issues = get_local_issues_without_matches(local_issues, &github_issues);

    // println!("{:#?}", local_issues);
    // if local_issues.is_empty() {
    //     println!("No local issues to create");
    // } else {
    //     gh.create_issue(
    //         local_issues[0].title.clone(),
    //         local_issues[0].text_body.clone(),
    //         local_issues[0].labels.clone(),
    //     )
    //     .await?;
    // }

    // println!("Total issues: {}", &github_issues.len());
    // // gh.close_all_issues().await?;
    cli()?;

    Ok(())
}
