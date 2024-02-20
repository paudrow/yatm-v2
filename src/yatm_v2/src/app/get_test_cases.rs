use crate::app::requirements::get_requirements_from_files;
use crate::app::test_cases_builder::get_test_cases_builders_from_files;
use crate::test_cases::make_test_cases;
use crate::types::Config;

use anyhow::{Context, Result};
use common::types::TestCase;

/// Get the test cases.
///
/// This combines the requirements and the test cases builders to create the test cases.
pub fn get_test_cases(config: &Config) -> Result<Vec<TestCase>> {
    let requirements = get_requirements_from_files(&config.requirements_dirs)
        .context("Failed to get requirements - before checking test cases")?;
    let test_cases_builders = get_test_cases_builders_from_files(&config.test_cases_builders_dirs)
        .context("Failed to get test case builder - before checking the test cases")?;
    let test_cases = make_test_cases(&test_cases_builders, &requirements);
    Ok(test_cases)
}
