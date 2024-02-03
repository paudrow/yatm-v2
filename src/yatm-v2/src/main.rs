mod requirement;
mod test_case;
mod test_cases_builder;
use std::collections::HashMap;

use itertools::Itertools;

use serde_yaml;

fn demo_requirements() {
    let r = requirement::Requirement {
        name: "Test requirement".to_string(),
        description: "My description".to_string(),
        labels: vec!["label1".to_string(), "label2".to_string()],
        steps: vec![requirement::Step {
            action: vec![
                requirement::Action::StdIn(requirement::Terminal {
                    number: 1,
                    text: "Test".to_string(),
                }),
                requirement::Action::Image("image".to_string()),
                requirement::Action::Describe("description".to_string()),
                requirement::Action::StdIn(requirement::Terminal {
                    number: 2,
                    text: "Test".to_string(),
                }),
            ],
            expect: vec![
                requirement::Expect::StdOut(requirement::Terminal {
                    number: 1,
                    text: "Test".to_string(),
                }),
                requirement::Expect::StdErr(requirement::Terminal {
                    number: 1,
                    text: "std err".to_string(),
                }),
            ],
        }],
    };

    let s = serde_yaml::to_string(&r).unwrap();
    println!("{}", s);

    let r2: requirement::Requirement = serde_yaml::from_str(&s).unwrap();
    println!("{:?}", r2);
}

fn demo_test_cases_builder() {
    let t = test_cases_builder::TestCasesBuilder {
        name: "Test test case".to_string(),
        description: "My description".to_string(),
        labels: vec!["label1".to_string(), "label2".to_string()],
        set: vec![
            test_cases_builder::SetSteps::Include(test_cases_builder::Filter {
                label: Some(vec!["label1".to_string()]),
                name: None,
                negate: false,
            }),
            test_cases_builder::SetSteps::Exclude(test_cases_builder::Filter {
                label: Some(vec!["label2".to_string()]),
                name: Some(vec!["name1".to_string(), "name2".to_string()]),
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

    let t2: test_cases_builder::TestCasesBuilder = serde_yaml::from_str(&s).unwrap();
    println!("{:?}", t2);
}

fn demo_test_case() {
    let t = test_case::TestCase {
        requirement: requirement::Requirement {
            name: "Test requirement".to_string(),
            description: "My description".to_string(),
            labels: vec!["label1".to_string(), "label2".to_string()],
            steps: vec![requirement::Step {
                action: vec![
                    requirement::Action::StdIn(requirement::Terminal {
                        number: 1,
                        text: "Test".to_string(),
                    }),
                    requirement::Action::Image("image".to_string()),
                    requirement::Action::Describe("description".to_string()),
                    requirement::Action::StdIn(requirement::Terminal {
                        number: 2,
                        text: "Test".to_string(),
                    }),
                ],
                expect: vec![
                    requirement::Expect::StdOut(requirement::Terminal {
                        number: 1,
                        text: "Test".to_string(),
                    }),
                    requirement::Expect::StdErr(requirement::Terminal {
                        number: 1,
                        text: "std err".to_string(),
                    }),
                ],
            }],
        },
        builder_used: test_cases_builder::TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: vec!["label1".to_string(), "label2".to_string()],
            set: vec![
                test_cases_builder::SetSteps::Include(test_cases_builder::Filter {
                    label: Some(vec!["label1".to_string()]),
                    name: None,
                    negate: false,
                }),
                test_cases_builder::SetSteps::Exclude(test_cases_builder::Filter {
                    label: Some(vec!["label2".to_string()]),
                    name: Some(vec!["name1".to_string(), "name2".to_string()]),
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

    let t2: test_case::TestCase = serde_yaml::from_str(&s).unwrap();
    println!("{:?}", t2);
}

fn get_cartesian_product(data: HashMap<String, Vec<String>>) -> Vec<HashMap<String, String>> {
    // Convert the HashMap into a Vec of (key, Vec<value>) pairs
    let mut items: Vec<(_, _)> = data.into_iter().collect();

    // Sort the items to ensure consistent ordering for the product
    items.sort_by_key(|t| t.0.clone());

    // Extract the keys and corresponding value iterators
    let keys: Vec<String> = items.iter().map(|t| t.0.clone()).collect();
    let value_iters: Vec<_> = items.into_iter().map(|(_, v)| v.into_iter()).collect();

    // Compute the Cartesian product of the value iterators
    value_iters
        .into_iter()
        .multi_cartesian_product()
        // Map each product to a HashMap
        .map(|values| {
            keys.clone()
                .into_iter()
                .zip(values)
                .collect::<HashMap<_, _>>()
        })
        .collect()
}

fn main() {
    // demo_requirements();
    // demo_test_cases_builder();
    // demo_test_case();

    let mut dimensions = HashMap::new();
    dimensions.insert(
        "color".to_string(),
        vec!["red".to_string(), "blue".to_string(), "green".to_string()],
    );
    dimensions.insert(
        "size".to_string(),
        vec!["small".to_string(), "large".to_string()],
    );
    dimensions.insert(
        "logo".to_string(),
        vec!["pika".to_string(), "poke".to_string()],
    );

    let product = get_cartesian_product(dimensions);
    for p in &product {
        println!("{:?}", p);
    }
}
