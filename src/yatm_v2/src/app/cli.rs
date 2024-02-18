use crate::app::load_config::load_config;
use crate::app::requirements_validate::validate_requirements;
use crate::app::{init_config::init_config, requirements_validate};

use std::path::PathBuf;

use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};

// Define the main application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct MyApp {
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
    /// Check the requirements
    Validate {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
    /// List the requirements
    List,
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
    let cli = MyApp::parse();

    match cli.command {
        Commands::Init { path } => {
            init_config(&path)?;
        }
        Commands::Requirements { subcommand } => match subcommand {
            RequirementsSubcommands::Validate { config_path } => {
                let config = load_config(&config_path)?;
                validate_requirements(&config.requirements_dirs)?;
                println!("Looks good 👍");
            }
            RequirementsSubcommands::List => {
                println!("Running list");
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
