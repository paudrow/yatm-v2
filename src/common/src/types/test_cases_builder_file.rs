use crate::types::TestCasesBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCasesBuilderFile {
    pub test_cases_builders: Vec<TestCasesBuilder>,
}

impl TestCasesBuilderFile {
    pub fn default() -> Self {
        TestCasesBuilderFile {
            test_cases_builders: vec![TestCasesBuilder::default()],
        }
    }
}
