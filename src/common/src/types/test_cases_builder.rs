use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCasesBuilder {
    pub name: String,
    pub description: String,
    pub set: Vec<SetSteps>,
    pub labels: Option<Vec<String>>,
    pub permutations: HashMap<String, Vec<String>>,
    pub version: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SetSteps {
    Include(Filter),
    Exclude(Filter),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Filter {
    pub all_labels: Option<Vec<String>>,
    pub any_names: Option<Vec<String>>,
    pub negate: bool,
}
