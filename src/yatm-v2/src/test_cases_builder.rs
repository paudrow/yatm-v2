use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCasesBuilder {
    pub name: String,
    pub description: String,
    pub labels: Vec<String>,
    pub set: Vec<SetSteps>,
    pub permutations: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SetSteps {
    Include(Filter),
    Exclude(Filter),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Filter {
    pub label: Option<Vec<String>>,
    pub name: Option<Vec<String>>,
    pub negate: bool,
}
