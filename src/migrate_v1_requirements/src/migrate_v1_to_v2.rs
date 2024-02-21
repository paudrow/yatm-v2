use crate::requirements_file_v1::{RequirementV1, RequirementsFileV1};
use anyhow::{Context, Result};
use common::types::{
    Action as ActionV2, Expect as ExpectV2, Requirement as RequirementV2,
    RequirementsFile as RequirementsFileV2, Step as StepV2, Terminal as TerminalV2,
};

pub fn convert_requirements_file_v1_to_v2(
    file_v1: &RequirementsFileV1,
) -> Result<RequirementsFileV2> {
    let requirements: Result<Vec<RequirementV2>, _> =
        file_v1.requirements.iter().map(convert_v1_to_v2).collect();
    let requirements = requirements.context("Failed to convert requirements from v1 to v2")?;
    Ok(RequirementsFileV2 { requirements })
}

fn convert_v1_to_v2(req_v1: &RequirementV1) -> Result<RequirementV2> {
    let steps = req_v1
        .checks
        .iter()
        .map(|check| {
            let actions = check
                .r#try
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|try_item| {
                    try_item.stdin.as_ref().map(|stdin| {
                        ActionV2::StdIn(TerminalV2 {
                            number: try_item.terminal.unwrap_or(1), // Default to terminal 1 if not specified
                            text: stdin.clone(),
                        })
                    })
                })
                .collect();

            let expects = check
                .expect
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .map(|expect_item| {
                    if let Some(stdout) = &expect_item.stdout {
                        ExpectV2::StdOut(TerminalV2 {
                            number: expect_item.terminal.unwrap_or(1), // Default to terminal 1 if not specified
                            text: stdout.clone(),
                        })
                    } else if let Some(stdin) = &expect_item.stdin {
                        ExpectV2::StdIn(TerminalV2 {
                            number: expect_item.terminal.unwrap_or(1), // Default to terminal 1 if not specified
                            text: stdin.clone(),
                        })
                    } else {
                        ExpectV2::Describe(expect_item.note.clone().unwrap_or_default())
                    }
                })
                .collect();

            StepV2 {
                action: actions,
                expect: expects,
            }
        })
        .collect();

    Ok(RequirementV2 {
        name: req_v1.name.clone(),
        description: req_v1.description.clone().unwrap_or_default(),
        steps,
        labels: req_v1.labels.clone(),
        links: req_v1.links.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::convert_requirements_file_v1_to_v2;
    use crate::requirements_file_v1::RequirementsFileV1;
    use std::path::Path;

    #[test]
    fn test_convert_v1_to_v2() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/test_data");
        for entry in std::fs::read_dir(&dir).expect("Reads the directory") {
            let entry = entry.expect("Reads the entry");
            let path = entry.path();
            let contents = std::fs::read_to_string(path).unwrap();
            let requirements: RequirementsFileV1 =
                serde_yaml::from_str(&contents).expect("Parses the YAML");
            convert_requirements_file_v1_to_v2(&requirements).expect("Converts to v2");
        }
    }
}
