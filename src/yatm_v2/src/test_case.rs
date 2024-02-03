use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::requirement::Requirement;
use crate::test_cases_builder::TestCasesBuilder;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
    pub requirement: Requirement,
    pub builder_used: TestCasesBuilder,
    pub selected_permutation: HashMap<String, String>,
}
