use common::types::{Requirement, Step, Action, Expect, Terminal, SetSteps, Filter, TestCasesBuilder, TestCase};
use std::collections::HashMap;

use serde_yaml;

mod utils;

fn demo_requirements() {
    let r = Requirement {
        name: "Test requirement".to_string(),
        description: "My description".to_string(),
        labels: vec!["label1".to_string(), "label2".to_string()],
        steps: vec![Step {
            action: vec![
                Action::StdIn(Terminal {
                    number: 1,
                    text: "Test".to_string(),
                }),
                Action::Image("image".to_string()),
                Action::Describe("description".to_string()),
                Action::StdIn(Terminal {
                    number: 2,
                    text: "Test".to_string(),
                }),
            ],
            expect: vec![
                Expect::StdOut(Terminal {
                    number: 1,
                    text: "Test".to_string(),
                }),
                Expect::StdErr(Terminal {
                    number: 1,
                    text: "std err".to_string(),
                }),
            ],
        }],
    };

    let s = serde_yaml::to_string(&r).unwrap();
    println!("{}", s);

    let r2: Requirement = serde_yaml::from_str(&s).unwrap();
    println!("{:?}", r2);
}

fn demo_test_cases_builder() {
    let t = TestCasesBuilder {
        name: "Test test case".to_string(),
        description: "My description".to_string(),
        labels: vec!["label1".to_string(), "label2".to_string()],
        set: vec![
            SetSteps::Include(Filter {
                all_labels: Some(vec!["label1".to_string()]),
                any_names: None,
                negate: false,
            }),
            SetSteps::Exclude(Filter {
                all_labels: Some(vec!["label2".to_string()]),
                any_names: Some(vec!["name1".to_string(), "name2".to_string()]),
                negate: false,
            }),
        ],
        permutations: {
            let mut m = std::collections::HashMap::new();
            m.insert(
                "key1".to_string(),
                vec!["value1".to_string(), "value2".to_string()],
            );
            m.insert(
                "key2".to_string(),
                vec!["value3".to_string(), "value4".to_string()],
            );
            m
        },
    };

    let s = serde_yaml::to_string(&t).unwrap();
    println!("{}", s);

    let t2: TestCasesBuilder = serde_yaml::from_str(&s).unwrap();
    println!("{:?}", t2);
}

fn demo_test_case() {
    let t = TestCase {
        requirement: Requirement {
            name: "Test requirement".to_string(),
            description: "My description".to_string(),
            labels: vec!["label1".to_string(), "label2".to_string()],
            steps: vec![Step {
                action: vec![
                    Action::StdIn(Terminal {
                        number: 1,
                        text: "Test".to_string(),
                    }),
                    Action::Image("image".to_string()),
                    Action::Describe("description".to_string()),
                    Action::StdIn(Terminal {
                        number: 2,
                        text: "Test".to_string(),
                    }),
                ],
                expect: vec![
                    Expect::StdOut(Terminal {
                        number: 1,
                        text: "Test".to_string(),
                    }),
                    Expect::StdErr(Terminal {
                        number: 1,
                        text: "std err".to_string(),
                    }),
                ],
            }],
        },
        builder_used: TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: vec!["label1".to_string(), "label2".to_string()],
            set: vec![
                SetSteps::Include(Filter {
                    all_labels: Some(vec!["label1".to_string()]),
                    any_names: None,
                    negate: false,
                }),
                SetSteps::Exclude(Filter {
                    all_labels: Some(vec!["label2".to_string()]),
                    any_names: Some(vec!["name1".to_string(), "name2".to_string()]),
                    negate: false,
                }),
            ],
            permutations: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "key1".to_string(),
                    vec!["value1".to_string(), "value2".to_string()],
                );
                m.insert(
                    "key2".to_string(),
                    vec!["value3".to_string(), "value4".to_string()],
                );
                m
            },
        },
        selected_permutation: {
            let mut m = std::collections::HashMap::new();
            m.insert("key1".to_string(), "value1".to_string());
            m.insert("key2".to_string(), "value3".to_string());
            m
        },
    };

    let s = serde_yaml::to_string(&t).unwrap();
    println!("{}", s);

    let t2: TestCase = serde_yaml::from_str(&s).unwrap();
    println!("{:?}", t2);
}


fn main() {
    demo_requirements();
    demo_test_cases_builder();
    demo_test_case();
}
