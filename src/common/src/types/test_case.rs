use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::requirement::Requirement;
use crate::types::test_cases_builder::TestCasesBuilder;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCase {
    pub requirement: Requirement,
    pub builder_used: TestCasesBuilder,
    pub selected_permutation: HashMap<String, String>,
}
