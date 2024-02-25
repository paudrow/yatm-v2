use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCasesBuilder {
    /// The name of the test case builder.
    pub name: String,
    /// The description of the test case builder.
    pub description: String,
    /// The set of steps to build the test cases.
    pub set: Vec<SetSteps>,
    /// The labels to apply to the test cases.
    pub labels: Option<Vec<String>>,
    /// The permutations to apply to the test cases.
    pub permutations: HashMap<String, Vec<String>>,
}

impl TestCasesBuilder {
    pub fn default() -> Self {
        let mut permutations = HashMap::new();
        permutations.insert(
            "Operating System".to_string(),
            vec![
                "Ubuntu 22.04".to_string(),
                "Windows 11".to_string(),
                "MacOS 12.0".to_string(),
            ],
        );
        permutations.insert(
            "RMW".to_string(),
            vec!["CycloneDDS".to_string(), "FastRTPS".to_string()],
        );

        TestCasesBuilder {
            name: "Demo test cases".to_string(),
            description: "description".to_string(),
            set: vec![
                SetSteps::Include(Filter {
                    all_labels: Some(vec!["label".to_string()]),
                    any_names: Some(vec!["name".to_string()]),
                    negate: false,
                }),
                SetSteps::Exclude(Filter {
                    all_labels: None,
                    any_names: Some(vec!["Demo".to_string()]),
                    negate: false,
                }),
            ],
            labels: Some(vec!["Demo".to_string()]),
            permutations,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SetSteps {
    /// Include requirements that match the filter
    Include(Filter),
    /// Exclude requirements that match the filter
    Exclude(Filter),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Filter {
    /// The labels to filter the requirements.
    pub all_labels: Option<Vec<String>>,
    /// The names to filter the requirements.
    pub any_names: Option<Vec<String>>,
    /// If the filter should negate the result.
    pub negate: bool,
}
