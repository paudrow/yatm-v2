use crate::requirements_file_v1::{CheckV1, RequirementV1, RequirementsFileV1, StepV1};
use anyhow::{Context, Result};
use common::types::{
    Action as ActionV2, Expect as ExpectV2, Requirement as RequirementV2,
    RequirementsFile as RequirementsFileV2, Step as StepV2, Terminal as TerminalV2,
};
use convert_case::{Case, Casing};

pub fn convert_requirements_file_v1_to_v2(
    file_v1: &RequirementsFileV1,
) -> Result<RequirementsFileV2> {
    let requirements: Result<Vec<RequirementV2>, _> =
        file_v1.requirements.iter().map(convert_v1_to_v2).collect();
    let requirements = requirements.context("Failed to convert requirements from v1 to v2")?;
    Ok(RequirementsFileV2 { requirements })
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

fn convert_v1_to_v2(req_v1: &RequirementV1) -> Result<RequirementV2> {
    let steps = req_v1
        .checks
        .iter()
        .map(check_v1_to_step_v2)
        .collect::<Result<Vec<StepV2>>>()?;

    Ok(RequirementV2 {
        name: req_v1.name.clone(),
        shortname: Some(req_v1.name.clone().to_case(Case::Snake)),
        description: req_v1.description.clone().unwrap_or_default(),
        steps,
        labels: req_v1.labels.clone(),
        links: req_v1.links.clone(),
    })
}

fn check_v1_to_step_v2(check: &CheckV1) -> Result<StepV2> {
    let actions = check
        .r#try
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .map(try_v1_to_actions_v2)
        .collect::<Result<Vec<Vec<ActionV2>>>>()?
        .into_iter()
        .flatten()
        .collect();

    let expects = check
        .expect
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .map(expect_v1_to_expects_v2)
        .collect::<Result<Vec<Vec<ExpectV2>>>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(StepV2 {
        name: Some(check.name.clone()),
        description: check.description.clone(),
        action: actions,
        expect: expects,
    })
}

fn try_v1_to_actions_v2(try_v1: &StepV1) -> Result<Vec<ActionV2>> {
    let mut actions = vec![];
    if let Some(note) = &try_v1.note {
        actions.push(ActionV2::Describe(note.clone()));
    }
    if let Some(stdin) = &try_v1.stdin {
        actions.push(ActionV2::StdIn(TerminalV2 {
            number: try_v1.terminal.unwrap_or(1), // Default to terminal 1 if not specified
            text: stdin.clone(),
        }));
    }
    if let Some(image_url) = &try_v1.imageUrl {
        actions.push(ActionV2::Image(image_url.clone()));
    }
    if let Some(stdout) = &try_v1.stdout {
        actions.push(ActionV2::Describe(format!("Stdout: {:?}", stdout.clone())));
    }
    if let Some(stderr) = &try_v1.stderr {
        actions.push(ActionV2::Describe(format!("Stderr: {:?}", stderr.clone())));
    }
    if actions.is_empty() {
        anyhow::bail!("There should be at least one action")
    }
    Ok(actions)
}

#[cfg(test)]
mod test_try_v1_to_actions_v2 {
    use super::try_v1_to_actions_v2;
    use crate::requirements_file_v1::StepV1;
    use common::types::{Action as ActionV2, Terminal as TerminalV2};

    #[test]
    fn test_try_v1_to_actions_v2() {
        let try_v1 = StepV1 {
            stdin: Some("echo hello".to_string()),
            note: Some("This is a note".to_string()),
            terminal: Some(2),
            imageUrl: Some("http://example.com".to_string()),
            stderr: None,
            stdout: None,
        };
        let actions = try_v1_to_actions_v2(&try_v1).expect("Converts to actions");

        assert_eq!(actions.len(), 3);
        for action in actions {
            match action {
                ActionV2::StdIn(TerminalV2 { number, text }) => {
                    assert_eq!(number, 2);
                    assert_eq!(text, "echo hello");
                }
                ActionV2::Describe(note) => {
                    assert_eq!(note, "This is a note");
                }
                ActionV2::Image(image_url) => {
                    assert_eq!(image_url, "http://example.com");
                }
                ActionV2::Url(_) => panic!("No corresponding v1 field"),
            }
        }
    }

    #[test]
    fn stdout_and_stderr_convert_to_describe() {
        let try_v1 = StepV1 {
            stdin: None,
            note: None,
            terminal: None,
            imageUrl: None,
            stderr: Some("error".to_string()),
            stdout: Some("hello".to_string()),
        };
        let actions = try_v1_to_actions_v2(&try_v1).expect("Converts to actions");

        assert_eq!(actions.len(), 2);
        for action in actions {
            match action {
                ActionV2::Describe(note) => {
                    assert!(note.contains("hello") || note.contains("error"));
                }
                _ => panic!("No corresponding v1 field"),
            }
        }
    }

    #[test]
    fn fail_if_no_actions() {
        let try_v1 = StepV1 {
            stdin: None,
            note: None,
            terminal: None,
            imageUrl: None,
            stderr: None,
            stdout: None,
        };
        let result = try_v1_to_actions_v2(&try_v1);
        assert!(result.is_err());
    }
}

fn expect_v1_to_expects_v2(expect_v1: &StepV1) -> Result<Vec<ExpectV2>> {
    let mut expects = vec![];

    if let Some(note) = &expect_v1.note {
        expects.push(ExpectV2::Describe(note.clone()));
    }
    if let Some(stdout) = &expect_v1.stdout {
        expects.push(ExpectV2::StdOut(TerminalV2 {
            number: expect_v1.terminal.unwrap_or(1), // Default to terminal 1 if not specified
            text: stdout.clone(),
        }));
    }
    if let Some(stderr) = &expect_v1.stderr {
        expects.push(ExpectV2::StdErr(TerminalV2 {
            number: expect_v1.terminal.unwrap_or(1), // Default to terminal 1 if not specified
            text: stderr.clone(),
        }));
    }
    if let Some(image_url) = &expect_v1.imageUrl {
        expects.push(ExpectV2::Image(image_url.clone()));
    }
    if let Some(stdin) = &expect_v1.stdin {
        expects.push(ExpectV2::Describe(format!("Stdin: {:?}", stdin.clone())));
    }
    if expects.is_empty() {
        anyhow::bail!("There should be at least one expect")
    }
    Ok(expects)
}

#[cfg(test)]
mod test_expect_v1_to_expects_v2 {

    use super::expect_v1_to_expects_v2;
    use crate::requirements_file_v1::StepV1;
    use common::types::{Expect as ExpectV2, Terminal as TerminalV2};

    #[test]
    fn test_expect_v1_to_expects_v2() {
        let expect_v1 = StepV1 {
            note: Some("This is a note".to_string()),
            imageUrl: Some("http://example.com".to_string()),
            stdin: None,
            stdout: Some("hello".to_string()),
            stderr: Some("".to_string()),
            terminal: Some(2),
        };
        let expects = expect_v1_to_expects_v2(&expect_v1).expect("Converts to expects");

        assert_eq!(expects.len(), 4);
        for expect in expects {
            match expect {
                ExpectV2::StdOut(TerminalV2 { number, text }) => {
                    assert_eq!(number, 2);
                    assert_eq!(text, "hello");
                }
                ExpectV2::Describe(note) => {
                    assert_eq!(note, "This is a note");
                }
                ExpectV2::Image(url) => {
                    assert_eq!(url, "http://example.com");
                }
                ExpectV2::StdErr(TerminalV2 { number, text }) => {
                    assert_eq!(number, 2);
                    assert_eq!(text, "");
                }
                ExpectV2::Url(_) => panic!("No corresponding v1 field"),
            }
        }
    }

    #[test]
    fn stdin_convert_to_describe() {
        let expect_v1 = StepV1 {
            note: None,
            imageUrl: None,
            stdin: Some("echo hello".to_string()),
            stdout: None,
            stderr: None,
            terminal: None,
        };
        let expects = expect_v1_to_expects_v2(&expect_v1).expect("Converts to expects");

        assert_eq!(expects.len(), 1);
        for expect in expects {
            match expect {
                ExpectV2::Describe(note) => {
                    assert!(note.contains("hello"));
                }
                _ => panic!("No corresponding v1 field"),
            }
        }
    }

    #[test]
    fn fail_if_no_expects() {
        let expect_v1 = StepV1 {
            stdout: None,
            stdin: None,
            note: None,
            terminal: None,
            imageUrl: None,
            stderr: None,
        };
        let result = expect_v1_to_expects_v2(&expect_v1);
        assert!(result.is_err());
    }
}
