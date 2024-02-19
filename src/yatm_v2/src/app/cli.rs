use crate::app::load_config::load_config;
use crate::app::requirements_validate::get_requirements_from_files;
use crate::app::{
    init_config::init_config,
    requirements_validate::{
        get_requirements_from_file, validate_requirements_file, validate_requirements_files,
    },
};
use crate::types::RequirementsFile;
use common::types::Requirement;

use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
use clap::{Parser, Subcommand};

use super::requirements_validate::get_requirements_files;

// Define the main application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct App {
    #[clap(subcommand)]
    pub command: Commands,
}

// Define the top-level subcommands
#[derive(Subcommand)]
enum Commands {
    /// Create a directory to start working from
    Init {
        /// The path to the project
        #[clap(short, long)]
        path: PathBuf,
    },
    /// Manage the requirements
    Requirements {
        #[clap(subcommand)]
        subcommand: RequirementsSubcommands,
    },
    /// Manage the test cases
    TestCases {
        #[clap(subcommand)]
        subcommand: TestCasesSubcommands,
    },
    /// Manage the test cases on Github
    Github {
        #[clap(subcommand)]
        subcommand: GithubSubcommands,
    },
}

// Define the subcommands for the FirstSubcommand
#[derive(Subcommand)]
enum RequirementsSubcommands {
    New {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
        #[clap(short, long)]
        file_name: Option<String>,
    },
    /// Check the requirements
    Validate {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },

    /// Validate a single requirements file
    ValidateFile {
        /// The path to the requirements file
        #[clap(short, long)]
        file_path: PathBuf,
    },

    /// List the requirements
    List {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
}

// Options specific to Subcommand1
#[derive(Parser)]
struct Subcommand1Options {
    #[clap(short, long)]
    option: String,
}

#[derive(Subcommand)]
enum TestCasesSubcommands {
    /// Check the test cases
    Validate,
    /// List the test cases
    List,
    /// Preview the test cases
    Preview,
}

#[derive(Subcommand)]
enum GithubSubcommands {
    /// Check the test cases
    Validate,
    /// List the test cases
    List,
    /// Preview the test cases
    Preview,
}

pub fn cli() -> Result<()> {
    let app = App::parse();
    match app.command {
        Commands::Init { path } => {
            init_config(&path)?;
            println!("Created the project in {:?}", path);
        }
        Commands::Requirements { subcommand } => match subcommand {
            RequirementsSubcommands::New {
                config_path,
                file_name,
            } => {
                let config = load_config(&config_path)?;

                let file_name = match file_name {
                    Some(file_name) => file_name,
                    None => {
                        // get datetime string in current timezone
                        let datetime_string =
                            chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
                        format!("requirements-{}.yaml", datetime_string)
                    }
                };

                if config.requirements_dirs.len() < 1 {
                    anyhow::bail!("No requirements directories found");
                }
                let requirements_file_path = config.requirements_dirs[0].join(file_name);
                if requirements_file_path.exists() {
                    anyhow::bail!(
                        "The requirements file already exists: {:?}",
                        requirements_file_path
                    );
                }
                let requirements_file = RequirementsFile::default();
                let requirements_file = serde_yaml::to_string(&requirements_file)
                    .context("Failed to turn requirement into a string")?;
                std::fs::write(&requirements_file_path, requirements_file).context(format!(
                    "Failed to write the requirements file: {:?}",
                    requirements_file_path
                ))?;
                println!(
                    "Created the requirements file: {:?}",
                    requirements_file_path
                );
            }
            RequirementsSubcommands::Validate { config_path } => {
                let config = load_config(&config_path)?;
                validate_requirements_files(&config.requirements_dirs)?;
                println!("Looks good 👍");
            }
            RequirementsSubcommands::ValidateFile { file_path } => {
                validate_requirements_file(&file_path)?;
                println!("Looks good 👍");
            }
            RequirementsSubcommands::List { config_path } => {
                let config = load_config(&config_path)?;
                let requirements_files = get_requirements_files(&config.requirements_dirs)?;
                for requirement_file in requirements_files {
                    println!(
                        "{}",
                        requirement_file
                            .to_str()
                            .context("Failed to convert the path to a string")?
                    );
                    let requirements = get_requirements_from_file(&requirement_file)?;
                    for requirement in requirements {
                        let labels_string = match requirement.labels {
                            Some(labels) => format!(" - labels: {}", labels.join(", ")),
                            None => "".to_string(),
                        };
                        println!(" * {}{}", requirement.name, labels_string);
                    }
                    println!();
                }
            }
        },
        Commands::TestCases { subcommand } => match subcommand {
            TestCasesSubcommands::Validate => {
                println!("Running validate");
            }
            TestCasesSubcommands::List => {
                println!("Running list");
            }
            TestCasesSubcommands::Preview => {
                println!("Running preview");
            }
        },
        Commands::Github { subcommand } => match subcommand {
            GithubSubcommands::Validate => {
                println!("Running validate");
            }
            GithubSubcommands::List => {
                println!("Running list");
            }
            GithubSubcommands::Preview => {
                println!("Running preview");
            }
        },
    }
    Ok(())
}

#[cfg(test)]
mod test_cli {
    use std::path::PathBuf;

    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use tempfile::tempdir;

    fn get_command() -> Command {
        Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
    }

    fn get_number_of_files_in_dir(dir: &PathBuf) -> usize {
        let entries = std::fs::read_dir(dir).unwrap();
        entries.count()
    }

    #[test]
    fn test_help() {
        for arg in &["-h", "--help"] {
            let mut cmd = get_command();
            cmd.arg(arg)
                .assert()
                .success()
                .stdout(predicate::str::contains("Usage"));
        }
    }

    #[test]
    fn full_requirements_test() {
        let dir = tempdir().unwrap().path().to_path_buf();

        // run the init command
        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success()
            .stdout(predicate::str::contains("Created the project in"));

        // check that files are generated correctly
        assert!(dir.join("config.yaml").is_file());
        assert!(dir.join(".generated_files").is_dir());
        assert!(dir.join(".gitignore").is_file());
        let requirements_dir = dir.join("requirements");
        assert!(requirements_dir.is_dir());
        assert_eq!(get_number_of_files_in_dir(&requirements_dir), 1);

        // run the requirements new command
        let new_requirements_file_name = "my-test-requirements.yaml";
        let mut cmd = get_command();
        cmd.args(&[
            "requirements",
            "new",
            "--config-path",
            dir.to_str().unwrap(),
            "--file-name",
            new_requirements_file_name,
        ])
        .assert()
        .success();
        let new_requirements_file_path = requirements_dir.join(new_requirements_file_name);
        assert!(new_requirements_file_path.is_file());
        assert_eq!(get_number_of_files_in_dir(&requirements_dir), 2);

        // run the requirements new command without a file name
        let mut cmd = get_command();
        cmd.args(&[
            "requirements",
            "new",
            "--config-path",
            dir.to_str().unwrap(),
        ])
        .assert()
        .success();
        assert_eq!(get_number_of_files_in_dir(&requirements_dir), 3);

        // run the requirements list command
        let mut cmd = get_command();
        cmd.args(&[
            "requirements",
            "list",
            "--config-path",
            dir.to_str().unwrap(),
        ])
        .assert()
        .success();

        // run the requirements validate command
        let mut cmd = get_command();
        cmd.args(&[
            "requirements",
            "validate",
            "--config-path",
            dir.to_str().unwrap(),
        ])
        .assert()
        .success();

        // run the requirements validate file command
        let mut cmd = get_command();
        cmd.args(&[
            "requirements",
            "validate-file",
            "--file-path",
            new_requirements_file_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    }
}
