use std::collections::HashMap;

use common::types::{Filter, Requirement, SetSteps, TestCase, TestCasesBuilder};
use itertools::Itertools;

pub fn make_test_cases(
    test_cases_builder: &Vec<TestCasesBuilder>,
    requirements: &Vec<Requirement>,
) -> Vec<TestCase> {
    let mut test_cases = Vec::new();
    for test_cases_builder in test_cases_builder.iter() {
        test_cases.append(&mut make_test_cases_helper(
            test_cases_builder,
            requirements,
        ));
    }
    test_cases
}

/// Returns a vector of test cases based on the test case builder and requirements
fn make_test_cases_helper(
    test_cases_builder: &TestCasesBuilder,
    requirements: &Vec<Requirement>,
) -> Vec<TestCase> {
    let permutations = get_cartesian_product(test_cases_builder.permutations.clone());
    let requirements = get_selected_requirements(requirements, test_cases_builder);

    let mut test_cases = Vec::new();

    for requirement in requirements.iter() {
        for permutation in &permutations {
            test_cases.push(TestCase {
                requirement: requirement.clone(),
                builder_used: test_cases_builder.clone(),
                selected_permutation: permutation.clone(),
            });
        }
    }

    test_cases
}

#[cfg(test)]
mod test_make_test_cases {
    use super::make_test_cases_helper;
    use common::types::{Filter, Requirement, SetSteps, TestCase, TestCasesBuilder};

    fn is_match_test_cases(actual: &Vec<TestCase>, expected: &Vec<TestCase>) -> bool {
        if actual.len() != expected.len() {
            return false;
        }
        for (a, e) in actual.iter().zip(expected.iter()) {
            if a.requirement.name != e.requirement.name {
                return false;
            }
            if a.builder_used.name != e.builder_used.name {
                return false;
            }
            if a.selected_permutation != e.selected_permutation {
                return false;
            }
        }
        true
    }

    #[test]
    fn make_test_cases_two_matches() {
        let requirements = vec![
            Requirement {
                name: "name1".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label1".to_string()]),
                links: None,
                steps: vec![],
            },
            Requirement {
                name: "name2".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label2".to_string()]),
                links: None,
                steps: vec![],
            },
        ];
        let test_cases_builder = TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            set: vec![SetSteps::Include(Filter {
                all_labels: None,
                any_names: Some(vec!["name1".to_string(), "name2".to_string()]),
                negate: false,
            })],
            permutations: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "key1".to_string(),
                    vec!["value1".to_string(), "value2".to_string()],
                );
                m
            },
        };
        let result = make_test_cases_helper(&test_cases_builder, &requirements);
        let expected = vec![
            TestCase {
                requirement: requirements[0].clone(),
                builder_used: test_cases_builder.clone(),
                selected_permutation: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("key1".to_string(), "value1".to_string());
                    m
                },
            },
            TestCase {
                requirement: requirements[0].clone(),
                builder_used: test_cases_builder.clone(),
                selected_permutation: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("key1".to_string(), "value2".to_string());
                    m
                },
            },
            TestCase {
                requirement: requirements[1].clone(),
                builder_used: test_cases_builder.clone(),
                selected_permutation: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("key1".to_string(), "value1".to_string());
                    m
                },
            },
            TestCase {
                requirement: requirements[1].clone(),
                builder_used: test_cases_builder.clone(),
                selected_permutation: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("key1".to_string(), "value2".to_string());
                    m
                },
            },
        ];
        assert!(is_match_test_cases(&result, &expected));
    }

    #[test]
    fn test_make_test_cases_one_match() {
        let requirements = vec![
            Requirement {
                name: "name1".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label1".to_string()]),
                links: None,
                steps: vec![],
            },
            Requirement {
                name: "name2".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label2".to_string()]),
                links: None,
                steps: vec![],
            },
        ];
        let test_cases_builder = TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            set: vec![
                SetSteps::Include(Filter {
                    all_labels: None,
                    any_names: Some(vec!["name1".to_string(), "name2".to_string()]),
                    negate: false,
                }),
                SetSteps::Exclude(Filter {
                    all_labels: Some(vec!["label2".to_string()]),
                    any_names: None,
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
        let result = make_test_cases_helper(&test_cases_builder, &requirements);
        let expected = vec![
            TestCase {
                requirement: requirements[0].clone(),
                builder_used: test_cases_builder.clone(),
                selected_permutation: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("key1".to_string(), "value1".to_string());
                    m.insert("key2".to_string(), "value3".to_string());
                    m
                },
            },
            TestCase {
                requirement: requirements[0].clone(),
                builder_used: test_cases_builder.clone(),
                selected_permutation: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("key1".to_string(), "value1".to_string());
                    m.insert("key2".to_string(), "value4".to_string());
                    m
                },
            },
            TestCase {
                requirement: requirements[0].clone(),
                builder_used: test_cases_builder.clone(),
                selected_permutation: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("key1".to_string(), "value2".to_string());
                    m.insert("key2".to_string(), "value3".to_string());
                    m
                },
            },
            TestCase {
                requirement: requirements[0].clone(),
                builder_used: test_cases_builder.clone(),
                selected_permutation: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("key1".to_string(), "value2".to_string());
                    m.insert("key2".to_string(), "value4".to_string());
                    m
                },
            },
        ];
        assert!(is_match_test_cases(&result, &expected));
    }

    #[test]
    fn test_make_test_cases_no_match() {
        let requirements = vec![
            Requirement {
                name: "name1".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label1".to_string()]),
                links: None,
                steps: vec![],
            },
            Requirement {
                name: "name2".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label2".to_string()]),
                links: None,
                steps: vec![],
            },
        ];
        let test_cases_builder = TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: Some(vec!["label3".to_string()]),
            set: vec![SetSteps::Include(Filter {
                all_labels: Some(vec!["label-none".to_string()]),
                any_names: None,
                negate: false,
            })],
            permutations: Default::default(),
        };
        let result = make_test_cases_helper(&test_cases_builder, &requirements);
        let expected: Vec<TestCase> = vec![];
        assert!(is_match_test_cases(&result, &expected));
    }
}

fn get_cartesian_product<T, U>(data: HashMap<T, Vec<U>>) -> Vec<HashMap<T, U>>
where
    T: Clone + Eq + std::hash::Hash + Ord,
    U: Clone,
{
    // Convert the HashMap into a Vec of (key, Vec<value>) pairs
    let mut items: Vec<(_, _)> = data.into_iter().collect();

    // Sort the items to ensure consistent ordering for the product
    items.sort_by_key(|t| t.0.clone());

    // Extract the keys and corresponding value iterators
    let keys: Vec<T> = items.iter().map(|t| t.0.clone()).collect();
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

#[cfg(test)]
mod test_get_cartesian_product {
    use super::get_cartesian_product;
    use std::collections::HashMap;

    #[test]
    fn test_get_cartesian_product() {
        let data: HashMap<&str, Vec<&str>> = [("a", vec!["1", "2"]), ("b", vec!["3", "4"])]
            .iter()
            .cloned()
            .collect();

        let result = get_cartesian_product(data);

        let expected: Vec<HashMap<&str, &str>> = vec![
            [("a", "1"), ("b", "3")].iter().cloned().collect(),
            [("a", "1"), ("b", "4")].iter().cloned().collect(),
            [("a", "2"), ("b", "3")].iter().cloned().collect(),
            [("a", "2"), ("b", "4")].iter().cloned().collect(),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_cartesian_product_empty() {
        let data: HashMap<&str, Vec<&str>> = HashMap::new();
        let result = get_cartesian_product(data);
        let expected: Vec<HashMap<&str, &str>> = vec![];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_cartesian_product_single() {
        let data: HashMap<&str, Vec<&str>> = [("a", vec!["1", "2"])].iter().cloned().collect();
        let result = get_cartesian_product(data);
        let expected: Vec<HashMap<&str, &str>> = vec![
            [("a", "1")].iter().cloned().collect(),
            [("a", "2")].iter().cloned().collect(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_cartesian_product_single_empty() {
        let data: HashMap<&str, Vec<&str>> = [("a", vec![])].iter().cloned().collect();
        let result = get_cartesian_product(data);
        let expected: Vec<HashMap<&str, &str>> = vec![];
        assert_eq!(result, expected);
    }
}

/// Returns the requirements that match the filters
fn get_selected_requirements(
    requirements: &Vec<Requirement>,
    test_cases_builder: &TestCasesBuilder,
) -> Vec<Requirement> {
    let mut selected_requirements: Vec<Requirement> = vec![];
    for set in test_cases_builder.set.iter() {
        match set {
            SetSteps::Include(filter) => {
                for requirement in requirements.iter() {
                    if filter_matches_requirement(filter, requirement) {
                        selected_requirements.push(requirement.clone());
                    }
                }
            }
            SetSteps::Exclude(filter) => {
                selected_requirements = selected_requirements
                    .into_iter()
                    .filter(|requirement| !filter_matches_requirement(filter, requirement))
                    .collect();
            }
        }
    }
    selected_requirements
}

#[cfg(test)]
mod test_get_selected_requirements {
    use super::get_selected_requirements;
    use common::types::{Filter, Requirement, SetSteps, TestCasesBuilder};

    fn is_match_requirements(actual: &Vec<Requirement>, expected: &Vec<Requirement>) -> bool {
        if actual.len() != expected.len() {
            return false;
        }
        for (a, e) in actual.iter().zip(expected.iter()) {
            if a.name != e.name {
                return false;
            }
        }
        true
    }

    #[test]
    fn include_no_match() {
        let requirements = vec![
            Requirement {
                name: "name1".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label1".to_string()]),
                links: None,
                steps: vec![],
            },
            Requirement {
                name: "name2".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label2".to_string()]),
                links: None,
                steps: vec![],
            },
        ];
        let test_cases_builder = TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: Some(vec!["label3".to_string()]),
            set: vec![SetSteps::Include(Filter {
                all_labels: Some(vec!["label-none".to_string()]),
                any_names: None,
                negate: false,
            })],
            permutations: Default::default(),
        };
        let result = get_selected_requirements(&requirements, &test_cases_builder);
        let expected: Vec<Requirement> = vec![];
        assert!(is_match_requirements(&result, &expected));
    }

    #[test]
    fn include_one_step() {
        let requirements = vec![
            Requirement {
                name: "name1".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label1".to_string()]),
                links: None,
                steps: vec![],
            },
            Requirement {
                name: "name2".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label2".to_string()]),
                links: None,
                steps: vec![],
            },
        ];
        let test_cases_builder = TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            set: vec![SetSteps::Include(Filter {
                all_labels: Some(vec!["label1".to_string()]),
                any_names: None,
                negate: false,
            })],
            permutations: Default::default(),
        };
        let result = get_selected_requirements(&requirements, &test_cases_builder);
        let expected = vec![requirements[0].clone()];
        assert!(is_match_requirements(&result, &expected));
    }

    #[test]
    fn include_two_steps() {
        let requirements = vec![
            Requirement {
                name: "name1".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label1".to_string()]),
                links: None,
                steps: vec![],
            },
            Requirement {
                name: "name2".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label2".to_string()]),
                links: None,
                steps: vec![],
            },
        ];
        let test_cases_builder = TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            set: vec![
                SetSteps::Include(Filter {
                    all_labels: Some(vec!["label1".to_string()]),
                    any_names: None,
                    negate: false,
                }),
                SetSteps::Include(Filter {
                    all_labels: None,
                    any_names: Some(vec!["name2".to_string()]),
                    negate: false,
                }),
            ],
            permutations: Default::default(),
        };
        let result = get_selected_requirements(&requirements, &test_cases_builder);
        let expected = requirements.clone();
        assert!(is_match_requirements(&result, &expected));
    }

    #[test]
    fn exclude_one_step() {
        let requirements = vec![
            Requirement {
                name: "name1".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label1".to_string()]),
                links: None,
                steps: vec![],
            },
            Requirement {
                name: "name2".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label2".to_string()]),
                links: None,
                steps: vec![],
            },
        ];
        let test_cases_builder = TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            set: vec![
                SetSteps::Include(Filter {
                    all_labels: None,
                    any_names: Some(vec!["name1".to_string(), "name2".to_string()]),
                    negate: false,
                }),
                SetSteps::Exclude(Filter {
                    all_labels: None,
                    any_names: Some(vec!["name2".to_string()]),
                    negate: false,
                }),
            ],
            permutations: Default::default(),
        };
        let result = get_selected_requirements(&requirements, &test_cases_builder);
        let expected = vec![requirements[0].clone()];
        assert!(is_match_requirements(&result, &expected));
    }

    #[test]
    fn exclude_two_steps() {
        let requirements = vec![
            Requirement {
                name: "name1".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label1".to_string()]),
                links: None,
                steps: vec![],
            },
            Requirement {
                name: "name2".to_string(),
                description: "description".to_string(),
                labels: Some(vec!["label2".to_string()]),
                links: None,
                steps: vec![],
            },
        ];
        let test_cases_builder = TestCasesBuilder {
            name: "Test test case".to_string(),
            description: "My description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            set: vec![
                SetSteps::Include(Filter {
                    all_labels: None,
                    any_names: Some(vec!["name1".to_string(), "name2".to_string()]),
                    negate: false,
                }),
                SetSteps::Exclude(Filter {
                    all_labels: None,
                    any_names: Some(vec!["name2".to_string()]),
                    negate: false,
                }),
                SetSteps::Exclude(Filter {
                    all_labels: Some(vec!["label1".to_string()]),
                    any_names: None,
                    negate: false,
                }),
            ],
            permutations: Default::default(),
        };
        let result = get_selected_requirements(&requirements, &test_cases_builder);
        let expected = vec![];
        assert!(is_match_requirements(&result, &expected));
    }
}

/// Returns true if the requirement matches the filter
fn filter_matches_requirement(filter: &Filter, requirement: &Requirement) -> bool {
    let label_matches = match &filter.all_labels {
        Some(labels) => {
            let mut all_labels_match = true;
            for label in labels.iter() {
                if let Some(requirement_labels) = &requirement.labels {
                    if !requirement_labels.contains(label) {
                        all_labels_match = false;
                        break;
                    }
                }
            }
            all_labels_match
        }
        None => true,
    };
    let name_matches = match &filter.any_names {
        Some(names) => {
            let mut any_name_matches = false;
            for name in names.iter() {
                if requirement
                    .name
                    .to_lowercase()
                    .contains(&name.to_lowercase())
                {
                    any_name_matches = true;
                    break;
                }
            }
            any_name_matches
        }
        None => true,
    };
    filter.negate ^ (label_matches && name_matches)
}

#[cfg(test)]
mod test_filter_matches_requirements {

    use super::filter_matches_requirement;
    use common::types::{Filter, Requirement};

    #[test]
    fn label() {
        let filter = Filter {
            all_labels: Some(vec!["label1".to_string()]),
            any_names: None,
            negate: false,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), true);
    }

    #[test]
    fn label_negate() {
        let filter = Filter {
            all_labels: Some(vec!["label1".to_string()]),
            any_names: None,
            negate: true,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), false);
    }

    #[test]
    fn name() {
        let filter = Filter {
            all_labels: None,
            any_names: Some(vec!["name1".to_string()]),
            negate: false,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), true);
    }

    #[test]
    fn name_partial_case_insensitive() {
        let filter = Filter {
            all_labels: None,
            any_names: Some(vec!["Me1".to_string()]),
            negate: false,
        };
        let requirement = Requirement {
            name: "name1-1".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), true);
    }

    #[test]
    fn name_negate() {
        let filter = Filter {
            all_labels: None,
            any_names: Some(vec!["name1".to_string()]),
            negate: true,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), false);
    }

    #[test]
    fn name_and_label() {
        let filter = Filter {
            all_labels: Some(vec!["label1".to_string()]),
            any_names: Some(vec!["name1".to_string()]),
            negate: false,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), true);
    }

    #[test]
    fn name_and_label_negate() {
        let filter = Filter {
            all_labels: Some(vec!["label1".to_string()]),
            any_names: Some(vec!["name1".to_string()]),
            negate: true,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), false);
    }

    #[test]
    fn labels_with_extra_requirement_labels() {
        let filter = Filter {
            all_labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            any_names: None,
            negate: false,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec![
                "label1".to_string(),
                "label2".to_string(),
                "label3".to_string(),
            ]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), true);
    }

    #[test]
    fn labels_with_extra_requirement_labels_negate() {
        let filter = Filter {
            all_labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            any_names: None,
            negate: true,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec![
                "label1".to_string(),
                "label2".to_string(),
                "label3".to_string(),
            ]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), false);
    }

    #[test]
    fn not_all_filter_labels_match() {
        let filter = Filter {
            all_labels: Some(vec!["label1".to_string(), "label3".to_string()]),
            any_names: None,
            negate: false,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), false);
    }

    #[test]
    fn not_all_labels_match_negate() {
        let filter = Filter {
            all_labels: Some(vec!["label1".to_string(), "label3".to_string()]),
            any_names: None,
            negate: true,
        };
        let requirement = Requirement {
            name: "name1".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), true);
    }

    #[test]
    fn one_name_of_many() {
        let filter = Filter {
            all_labels: None,
            any_names: Some(vec!["name1".to_string(), "name3".to_string()]),
            negate: false,
        };
        let requirement = Requirement {
            name: "name3".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), true);
    }

    #[test]
    fn one_name_of_many_negate() {
        let filter = Filter {
            all_labels: None,
            any_names: Some(vec!["name1".to_string(), "name3".to_string()]),
            negate: true,
        };
        let requirement = Requirement {
            name: "name3".to_string(),
            description: "description".to_string(),
            labels: Some(vec!["label1".to_string(), "label2".to_string()]),
            links: None,
            steps: vec![],
        };
        assert_eq!(filter_matches_requirement(&filter, &requirement), false);
    }
}
