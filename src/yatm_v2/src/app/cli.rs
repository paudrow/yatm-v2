use crate::app::init_workspace::init_workspace;
use crate::app::load_config::load_config;
use crate::constants::YAML_EXTENSIONS;
use crate::helpers::{
    get_files, get_local_issues_matches, get_requirements_from_file, get_test_cases,
    get_test_cases_builders_from_file, permutation_to_labels, project_version_to_label,
    test_case_to_markdown, validate_requirements_file, validate_requirements_files,
    validate_test_cases_builder_file, GithubIssueMatches, IssueMatchType,
};
use crate::types::LocalIssue;
use common::github::Github;
use common::markdown_toc::{prepend_markdown_table_of_contents, TocOptions};
use common::types::{Link, RequirementsFile, TestCasesBuilderFile};

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
use askama::Template;
use clap::{Parser, Subcommand};
use octocrab::models::IssueState;
use octocrab::params::State;

#[derive(Template)]
#[template(path = "metrics_report.md", escape = "none")]
struct MetricsReportTemplate {
    total_issues: usize,
    open_count: usize,
    open_pct: String,
    completed_count: usize,
    completed_pct: String,
    wont_fix_count: usize,
    wont_fix_pct: String,
    duplicate_count: usize,
    duplicate_pct: String,
    permutations: Vec<PermutationKeyBreakdown>,
    pairwise_matrices: Vec<PairwiseMatrix>,
}

struct PermutationKeyBreakdown {
    key: String,
    values: Vec<PermutationValueBreakdown>,
}

struct PermutationValueBreakdown {
    value: String,
    closed_count: usize,
    total_count: usize,
    closed_pct: String,
    open_count: usize,
    open_pct: String,
    completed_count: usize,
    completed_pct: String,
    wont_fix_count: usize,
    wont_fix_pct: String,
    duplicate_count: usize,
    duplicate_pct: String,
    bar_width_pct: String,
    has_completed: bool,
    has_open: bool,
    has_wont_fix: bool,
    has_duplicate: bool,
}

struct PairwiseMatrix {
    key_a: String,
    key_b: String,
    headers: Vec<MatrixHeader>,
    rows: Vec<MatrixRow>,
}

struct MatrixHeader {
    value: String,
    width_pct: String,
}

struct MatrixRow {
    val_a: String,
    height: String,
    cells: Vec<MatrixCell>,
}

struct MatrixCell {
    total_cases: usize,
    completed: usize,
    total_valid: usize,
    completed_pct: String,
    open: usize,
    open_pct: String,
    wont_fix: usize,
    wont_fix_pct: String,
    duplicate: usize,
    duplicate_pct: String,
    has_mini_bar: bool,
    has_completed: bool,
    has_open: bool,
    has_wont_fix: bool,
    has_duplicate: bool,
    lightness: String,
}

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
    /// Create a new YATM workspace
    Init {
        /// The path to the new workspace
        #[clap(short, long)]
        path: PathBuf,
    },
    /// Manage the test cases on Github
    Github {
        #[clap(subcommand)]
        subcommand: GithubSubcommands,
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
}

// Define the subcommands for the FirstSubcommand
#[derive(Subcommand)]
enum RequirementsSubcommands {
    /// List the requirements
    List {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
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
}

// Options specific to Subcommand1
#[derive(Parser)]
struct Subcommand1Options {
    #[clap(short, long)]
    option: String,
}

#[derive(Subcommand)]
enum TestCasesSubcommands {
    /// List the test cases builder files
    List {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
    /// Create a new test case builder
    New {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
        #[clap(short, long)]
        file_name: Option<String>,
    },
    /// Preview the test cases in yaml.
    Preview {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
    /// Check the test cases builder files
    Validate {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },

    /// Validate a test cases builder file
    ValidateFile {
        /// The path to the test cases builder file
        #[clap(short, long)]
        file_path: PathBuf,
    },
}

#[derive(Subcommand)]
enum GithubSubcommands {
    /// Make links to the Github labels
    MakeLabelLinks {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
    /// Get metrics for the project on Github
    Metrics {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
        /// The label to analyze
        #[clap(short, long)]
        label: Option<String>,
        /// Generates a visual pairwise permutation matrix report to a file
        #[clap(short, long)]
        report: Option<PathBuf>,
    },
    /// Preview the test cases in markdown
    Preview {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
    /// Upload the test cases to Github
    Upload {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,

        /// Do not push things remotely
        #[clap(short = 'n', long)]
        dry_run: bool,
    },
    /// Utilities for Github
    Utils {
        #[clap(subcommand)]
        subcommand: GithubUtilsSubcommands,
    },
    /// Check the test cases are converted to markdown correctly.
    Validate {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
}

#[derive(Subcommand)]
enum GithubUtilsSubcommands {
    /// Close issues that have a matching label
    CloseIssues {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
        /// The label to close
        #[clap(short, long)]
        label: String,
        /// Don't ask for confirmation
        #[clap(short, long, action = clap::ArgAction::SetTrue)]
        yes: bool,
    },
    /// Create labels from the config
    CreateLabels {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
    /// Delete all existing labels
    DeleteAllLabels {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
        /// Don't ask for confirmation
        #[clap(short, long, action = clap::ArgAction::SetTrue)]
        yes: bool,
    },
    /// List the labels in the repo
    ListLabels {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
    },
}

pub async fn cli() -> Result<()> {
    let app = App::parse();
    match app.command {
        Commands::Init { path } => {
            init_workspace(&path)?;
            println!("Created a YATM workspace in {:?}", path);
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
                        let datetime_string =
                            chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
                        format!("requirements-{}.yaml", datetime_string)
                    }
                };

                let requirements_file_path = config.new_requirements_dir.join(file_name);
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
                let requirements_files = get_files(&config.requirements_dirs, &YAML_EXTENSIONS)?;
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
            TestCasesSubcommands::New {
                config_path,
                file_name,
            } => {
                let config = load_config(&config_path)?;

                let file_name = match file_name {
                    Some(file_name) => file_name,
                    None => {
                        let datetime_string =
                            chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
                        format!("test_cases_builder-{}.yaml", datetime_string)
                    }
                };

                let test_cases_builder_file_path =
                    config.new_test_cases_builder_dir.join(file_name);
                if test_cases_builder_file_path.exists() {
                    anyhow::bail!(
                        "The test cases builder file already exists: {:?}",
                        test_cases_builder_file_path
                    );
                }
                let test_cases_builder_file = TestCasesBuilderFile::default();
                let test_cases_builder_file = serde_yaml::to_string(&test_cases_builder_file)
                    .context("Failed to turn test cases builder file into a string")?;
                std::fs::write(&test_cases_builder_file_path, test_cases_builder_file).context(
                    format!(
                        "Failed to write the test cases builder file: {:?}",
                        test_cases_builder_file_path
                    ),
                )?;
                println!(
                    "Created the test cases builder file: {:?}",
                    test_cases_builder_file_path
                );
            }
            TestCasesSubcommands::Validate { config_path } => {
                let config = load_config(&config_path)?;
                let test_cases = get_test_cases(&config)?;
                if test_cases.is_empty() {
                    anyhow::bail!("No test cases would be created");
                }
                let test_cases_count = test_cases.len();
                println!("{} test cases would be made", test_cases_count);
                println!("Looks good 👍");
            }
            TestCasesSubcommands::ValidateFile { file_path } => {
                validate_test_cases_builder_file(&file_path)?;
                println!("Looks good 👍");
            }
            TestCasesSubcommands::List { config_path } => {
                let config = load_config(&config_path)?;
                let test_case_builder_files =
                    get_files(&config.test_cases_builders_dirs, &YAML_EXTENSIONS)?;
                for test_case_builder_file in test_case_builder_files {
                    println!(
                        "{}",
                        test_case_builder_file
                            .to_str()
                            .context("Failed to convert the path to a string")?
                    );
                    let test_cases_builders =
                        get_test_cases_builders_from_file(&test_case_builder_file)?;
                    for test_case in test_cases_builders {
                        println!(" * {}", test_case.name);
                    }
                    println!();
                }
            }
            TestCasesSubcommands::Preview { config_path } => {
                let config = load_config(&config_path)?;
                let test_cases = get_test_cases(&config)?;
                if test_cases.is_empty() {
                    anyhow::bail!("No test cases would be created");
                }

                let datetime_string = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
                let output_file_name = format!("test-cases-{}.yaml", datetime_string);
                let output_path = config.generated_files_dir.join(output_file_name);

                let test_cases = serde_yaml::to_string(&test_cases)
                    .context("Failed to turn test cases into a string")?;
                std::fs::write(&output_path, test_cases).context(format!(
                    "Failed to write the test cases file: {:?}",
                    output_path
                ))?;
                println!("Created the test cases preview file: {:?}", output_path);
            }
        },
        Commands::Github { subcommand } => match subcommand {
            GithubSubcommands::Validate { config_path } => {
                let config = load_config(&config_path)?;

                // Get the test cases
                let test_cases = get_test_cases(&config)?;
                if test_cases.is_empty() {
                    anyhow::bail!("No test cases found");
                }

                // Convert the test cases to markdown
                for test_case in &test_cases {
                    test_case_to_markdown(test_case.clone(), &config.workspace_version).context(
                        format!("Failed to convert test case to markdown: {:?}", &test_case),
                    )?;
                }
                let number_of_test_cases = test_cases.len();
                println!("{} test cases would be made", number_of_test_cases);
                println!("Looks good 👍");
            }
            GithubSubcommands::Preview { config_path } => {
                let config = load_config(&config_path)?;

                // Get the test cases
                let test_cases = get_test_cases(&config)?;
                if test_cases.is_empty() {
                    anyhow::bail!("No test cases found");
                }

                // Convert the test cases to markdown
                let mut file_contents = String::new();
                for test_case in test_cases {
                    let issue =
                        test_case_to_markdown(test_case.clone(), &config.workspace_version)?;

                    // Get the title with the selected permutation
                    let mut permutation =
                        test_case.selected_permutation.values().collect::<Vec<_>>();
                    permutation.sort();
                    let permutation = permutation
                        .iter()
                        .map(|v| format!("`{}`", v))
                        .collect::<Vec<_>>();
                    let title = format!("{} - {}", issue.title, permutation.join(", "),);

                    // Get the labels
                    let mut labels = issue
                        .labels
                        .iter()
                        .map(|l| format!("`{}`", l))
                        .collect::<Vec<_>>();
                    labels.sort();

                    let issue = format!(
                        "# {title}\n\nLabels: {labels}\n\n{text_body}\n\n---\n\n",
                        title = title,
                        labels = labels.join(", "),
                        text_body = issue.text_body,
                    );

                    file_contents.push_str(&issue);
                }

                let toc_options = TocOptions {
                    min_depth: Some(1),
                    max_depth: Some(1),
                    spaces_per_indent: Some(2),
                    toc_title: Some("Table of Contents".to_string()),
                    toc_title_level: Some(1),
                };
                file_contents =
                    prepend_markdown_table_of_contents(&file_contents, Some(&toc_options));
                let gh = Github::new(&config.repo_owner, &config.repo_name)?;

                file_contents = gh.prepend_github_info(&file_contents);

                // Write the test cases to a file
                let datetime_string = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
                let output_file_name = format!(
                    "config_path--{}--generated-test-cases--{}.md",
                    &config_path
                        .file_stem()
                        .unwrap_or(OsStr::new("undefined"))
                        .display(),
                    datetime_string
                );
                let output_path = config.generated_files_dir.join(output_file_name);
                std::fs::create_dir_all(&config.generated_files_dir).context(format!(
                    "Failed to create generated files dir: {:?}",
                    config.generated_files_dir
                ))?;
                std::fs::write(&output_path, file_contents).context(format!(
                    "Failed to write the test cases file: {:?}",
                    output_path
                ))?;
                println!("Created the test cases preview file: {:?}", output_path);
            }
            GithubSubcommands::Upload {
                config_path,
                dry_run,
            } => {
                let config = load_config(&config_path)?;

                // Get the test cases
                let test_cases = get_test_cases(&config)?;
                if test_cases.is_empty() {
                    anyhow::bail!("No test cases found");
                }

                // Convert the test cases to markdown
                let local_issues = test_cases
                    .iter()
                    .map(|test_case| {
                        test_case_to_markdown(test_case.clone(), &config.workspace_version).context(
                            format!("Failed to convert test case to markdown: {:?}", test_case),
                        )
                    })
                    .collect::<Result<Vec<_>>>()?;
                let local_issues_count = local_issues.len();
                println!("{} test cases total", local_issues_count);

                // Get the issues from Github
                let gh = Github::new(&config.repo_owner, &config.repo_name)?;
                println!(
                    "Connecting to repository: {}/{}",
                    config.repo_owner, config.repo_name
                );
                let github_issues = gh.get_issues(Some(State::All)).await?;

                let matched_issues = get_local_issues_matches(&local_issues, &github_issues);
                for i in &matched_issues {
                    match i.match_type {
                        IssueMatchType::Match => println!(
                            "Matched: {} -- {}",
                            i.local_issue.title,
                            i.github_issue
                                .as_ref()
                                .map_or("No URL for issue".to_string(), |issue| issue
                                    .html_url
                                    .to_string())
                        ),
                        IssueMatchType::MatchedWithDiff => println!(
                            "Changed: {} -- {}",
                            i.local_issue.title,
                            i.github_issue
                                .as_ref()
                                .map_or("No URL for issue".to_string(), |issue| issue
                                    .html_url
                                    .to_string())
                        ),
                        IssueMatchType::Missing => println!("Missing: {}", i.local_issue.title),
                        #[allow(unreachable_patterns)]
                        _ => println!("Unknown match type for {}", i.local_issue.title),
                    }
                }

                // Create issues that don't exist on Github
                let unmatched_local_issues: Vec<LocalIssue> = matched_issues
                    .iter()
                    .filter(|m| m.match_type == IssueMatchType::Missing)
                    .map(|m| m.local_issue.clone())
                    .collect();
                let unmatched_local_issues_count = unmatched_local_issues.len();
                println!("{} test cases without issues", unmatched_local_issues_count);
                for i in &unmatched_local_issues {
                    println!("Unmatched: {}", i.title);
                }
                for issue in unmatched_local_issues {
                    println!("Creating issue: {}", issue.title);
                    if dry_run {
                        println!("Dry run skipping update.");
                    } else {
                        gh.create_issue(issue.title, issue.text_body, issue.labels)
                            .await?;
                    }
                }
                println!("{} issues created", unmatched_local_issues_count);

                // Edit  issues that don't exactly match
                let changed_issues: Vec<&GithubIssueMatches> = matched_issues
                    .iter()
                    .filter(|m| m.match_type == IssueMatchType::MatchedWithDiff)
                    .collect();
                let changed_local_issues_count = changed_issues.len();
                println!(
                    "{} test cases with changed issues",
                    changed_local_issues_count
                );

                for m in &changed_issues {
                    println!(
                        "Updating issue: {}, {}",
                        m.local_issue.title,
                        m.github_issue.as_ref().unwrap().html_url
                    );
                    if dry_run {
                        println!("Dry run skipping update.");
                    } else {
                        gh.update_issue(
                            m.github_issue.clone().unwrap().number,
                            m.local_issue.title.clone(),
                            m.local_issue.text_body.clone(),
                        )
                        .await?; //, issue.labels
                    }
                }

                println!("Done 🚀");
            }
            GithubSubcommands::MakeLabelLinks { config_path } => {
                let config = load_config(&config_path)?;
                let test_cases = get_test_cases(&config)?;
                let mut permutations: HashSet<Vec<String>> = HashSet::new();
                for test_case in &test_cases {
                    permutations.insert(permutation_to_labels(&test_case.selected_permutation));
                }
                let mut links: Vec<Link> = vec![];
                for permutation in permutations {
                    let mut url = format!(
                        "https://github.com/{}/{}/issues?q=is:issue+is:open",
                        config.repo_owner, config.repo_name
                    );
                    for label in &permutation {
                        url += &format!("+label:%22{}%22", label.replace(" ", "+"));
                    }
                    let text = &permutation
                        .iter()
                        .map(|l| format!("`{}`", l))
                        .collect::<Vec<_>>()
                        .join(", ");
                    links.push(Link {
                        url,
                        name: text.to_string(),
                    });
                }

                let mut file_contents = String::new();
                file_contents.push_str("# Links to the Github labels\n\n");
                for link in links {
                    file_contents.push_str(&format!("- [{}]({})\n", link.name, link.url));
                }

                // Write the test cases to a file
                let datetime_string = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
                let output_file_name = format!("label-links-{}.md", datetime_string);
                let output_path = config.generated_files_dir.join(output_file_name);
                std::fs::write(&output_path, file_contents).context(format!(
                    "Failed to write the label links file: {:?}",
                    output_path
                ))?;
            }
            GithubSubcommands::Metrics {
                config_path,
                label,
                report,
            } => {
                let config = load_config(&config_path)?;
                let project_version = project_version_to_label(&config.workspace_version);

                let gh = Github::new(&config.repo_owner, &config.repo_name)?;

                let issues = gh.get_issues(Some(State::All)).await?;
                let issues = issues
                    .iter()
                    .filter(|i| {
                        let is_version = i
                            .labels
                            .iter()
                            .any(|gh_label| gh_label.name == project_version);
                        let is_label_of_interest = match &label {
                            Some(label) => i.labels.iter().any(|l| l.name == *label),
                            None => true,
                        };
                        is_version && is_label_of_interest
                    })
                    .collect::<Vec<_>>();
                if issues.is_empty() {
                    println!("No issues found");
                } else {
                    let closed_issues = issues
                        .iter()
                        .filter(|i| i.state == IssueState::Closed)
                        .collect::<Vec<_>>();

                    let closed_completed = closed_issues
                        .iter()
                        .filter(|i| {
                            let is_wont_fix = i.state_reason
                                == Some(octocrab::models::issues::IssueStateReason::NotPlanned);
                            let is_duplicate = i
                                .labels
                                .iter()
                                .any(|l| l.name.to_lowercase() == "duplicate");
                            !is_wont_fix && !is_duplicate
                        })
                        .collect::<Vec<_>>();

                    let closed_wont_fix = closed_issues
                        .iter()
                        .filter(|i| {
                            i.state_reason
                                == Some(octocrab::models::issues::IssueStateReason::NotPlanned)
                        })
                        .collect::<Vec<_>>();

                    let closed_duplicate = closed_issues
                        .iter()
                        .filter(|i| {
                            i.labels
                                .iter()
                                .any(|l| l.name.to_lowercase() == "duplicate")
                        })
                        .collect::<Vec<_>>();

                    let open_issues = issues
                        .iter()
                        .filter(|i| i.state == IssueState::Open)
                        .collect::<Vec<_>>();

                    println!(
                        "Overall Metrics:\n----------------------------------\nTotal Issues: {}\nOpen: {} ({:.2}%)\nClosed Completed: {} ({:.2}%)\nClosed Won't Fix: {} ({:.2}%)\nClosed Duplicate: {} ({:.2}%)",
                        issues.len(),
                        open_issues.len(),
                        (open_issues.len() as f64 / issues.len() as f64) * 100.0,
                        closed_completed.len(),
                        (closed_completed.len() as f64 / issues.len() as f64) * 100.0,
                        closed_wont_fix.len(),
                        (closed_wont_fix.len() as f64 / issues.len() as f64) * 100.0,
                        closed_duplicate.len(),
                        (closed_duplicate.len() as f64 / issues.len() as f64) * 100.0
                    );

                    let test_cases = get_test_cases(&config)?;
                    let mut permutation_keys_values: std::collections::BTreeMap<
                        String,
                        std::collections::BTreeSet<String>,
                    > = std::collections::BTreeMap::new();
                    for tc in &test_cases {
                        for (k, v) in &tc.selected_permutation {
                            permutation_keys_values
                                .entry(k.clone())
                                .or_default()
                                .insert(v.clone());
                        }
                    }

                    if !permutation_keys_values.is_empty() {
                        println!("\nBreakdown by Permutation Key/Value:");
                        println!("----------------------------------");
                        for (key, values) in &permutation_keys_values {
                            println!("{}:", key);
                            for value in values {
                                let label_str =
                                    crate::helpers::sanitize_label(format!("{}: {}", key, value));

                                let term_issues = issues
                                    .iter()
                                    .filter(|i| i.labels.iter().any(|l| l.name == label_str))
                                    .collect::<Vec<_>>();

                                if term_issues.is_empty() {
                                    continue;
                                }

                                let term_closed = term_issues
                                    .iter()
                                    .filter(|i| i.state == IssueState::Closed)
                                    .collect::<Vec<_>>();

                                let term_open = term_issues
                                    .iter()
                                    .filter(|i| i.state == IssueState::Open)
                                    .collect::<Vec<_>>();

                                let term_completed = term_closed
                                    .iter()
                                    .filter(|i| {
                                        let is_wont_fix = i.state_reason == Some(octocrab::models::issues::IssueStateReason::NotPlanned);
                                        let is_duplicate = i.labels.iter().any(|l| l.name.to_lowercase() == "duplicate");
                                        !is_wont_fix && !is_duplicate
                                    })
                                    .collect::<Vec<_>>();

                                let term_wont_fix = term_closed
                                    .iter()
                                    .filter(|i| i.state_reason == Some(octocrab::models::issues::IssueStateReason::NotPlanned))
                                    .collect::<Vec<_>>();

                                let term_duplicate = term_closed
                                    .iter()
                                    .filter(|i| {
                                        i.labels
                                            .iter()
                                            .any(|l| l.name.to_lowercase() == "duplicate")
                                    })
                                    .collect::<Vec<_>>();

                                println!(
                                    "  - {}: {}/{} closed ({:.2}%) -- Open: {} ({:.2}%), Closed Completed: {} ({:.2}%), Won't Fix: {} ({:.2}%), Duplicate: {} ({:.2}%)",
                                    value,
                                    term_closed.len(),
                                    term_issues.len(),
                                    (term_closed.len() as f64 / term_issues.len() as f64) * 100.0,
                                    term_open.len(),
                                    (term_open.len() as f64 / term_issues.len() as f64) * 100.0,
                                    term_completed.len(),
                                    (term_completed.len() as f64 / term_issues.len() as f64) * 100.0,
                                    term_wont_fix.len(),
                                    (term_wont_fix.len() as f64 / term_issues.len() as f64) * 100.0,
                                    term_duplicate.len(),
                                    (term_duplicate.len() as f64 / term_issues.len() as f64) * 100.0
                                );
                            }
                        }
                         if let Some(report_path) = &report {
                        let total_issues = issues.len();
                        let open_count = open_issues.len();
                        let open_pct = format!("{:.2}", (open_issues.len() as f64 / issues.len() as f64) * 100.0);
                        let completed_count = closed_completed.len();
                        let completed_pct = format!("{:.2}", (closed_completed.len() as f64 / issues.len() as f64) * 100.0);
                        let wont_fix_count = closed_wont_fix.len();
                        let wont_fix_pct = format!("{:.2}", (closed_wont_fix.len() as f64 / issues.len() as f64) * 100.0);
                        let duplicate_count = closed_duplicate.len();
                        let duplicate_pct = format!("{:.2}", (closed_duplicate.len() as f64 / issues.len() as f64) * 100.0);

                        let mut max_term_issues = 0;
                        if !permutation_keys_values.is_empty() {
                            for (key, values) in &permutation_keys_values {
                                for value in values {
                                    let label_str = crate::helpers::sanitize_label(format!(
                                        "{}: {}",
                                        key, value
                                    ));
                                    let cnt = issues
                                        .iter()
                                        .filter(|i| i.labels.iter().any(|l| l.name == label_str))
                                        .count();
                                    if cnt > max_term_issues {
                                        max_term_issues = cnt;
                                    }
                                }
                            }
                        }

                        let mut permutations = vec![];
                        if !permutation_keys_values.is_empty() {
                            for (key, values) in &permutation_keys_values {
                                let mut value_breakdowns = vec![];
                                for value in values {
                                    let label_str = crate::helpers::sanitize_label(format!(
                                        "{}: {}",
                                        key, value
                                    ));

                                    let term_issues = issues
                                        .iter()
                                        .filter(|i| i.labels.iter().any(|l| l.name == label_str))
                                        .collect::<Vec<_>>();

                                    if term_issues.is_empty() {
                                        continue;
                                    }

                                    let term_closed = term_issues
                                        .iter()
                                        .filter(|i| i.state == IssueState::Closed)
                                        .collect::<Vec<_>>();

                                    let term_open = term_issues
                                        .iter()
                                        .filter(|i| i.state == IssueState::Open)
                                        .collect::<Vec<_>>();

                                    let term_completed = term_closed
                                        .iter()
                                        .filter(|i| {
                                            let is_wont_fix = i.state_reason == Some(octocrab::models::issues::IssueStateReason::NotPlanned);
                                            let is_duplicate = i.labels.iter().any(|l| l.name.to_lowercase() == "duplicate");
                                            !is_wont_fix && !is_duplicate
                                        })
                                        .collect::<Vec<_>>();

                                    let term_wont_fix = term_closed
                                        .iter()
                                        .filter(|i| i.state_reason == Some(octocrab::models::issues::IssueStateReason::NotPlanned))
                                        .collect::<Vec<_>>();

                                    let term_duplicate = term_closed
                                        .iter()
                                        .filter(|i| {
                                            i.labels
                                                .iter()
                                                .any(|l| l.name.to_lowercase() == "duplicate")
                                        })
                                        .collect::<Vec<_>>();

                                    let closed_pct = (term_closed.len() as f64
                                        / term_issues.len() as f64)
                                        * 100.0;
                                    let open_pct =
                                        (term_open.len() as f64 / term_issues.len() as f64) * 100.0;
                                    let completed_pct = (term_completed.len() as f64
                                        / term_issues.len() as f64)
                                        * 100.0;
                                    let wont_fix_pct = (term_wont_fix.len() as f64
                                        / term_issues.len() as f64)
                                        * 100.0;
                                    let duplicate_pct = (term_duplicate.len() as f64
                                        / term_issues.len() as f64)
                                        * 100.0;

                                    let bar_width_pct = if max_term_issues > 0 {
                                        (term_issues.len() as f64 / max_term_issues as f64) * 100.0
                                    } else {
                                        0.0
                                    };

                                    value_breakdowns.push(PermutationValueBreakdown {
                                        value: value.clone(),
                                        closed_count: term_closed.len(),
                                        total_count: term_issues.len(),
                                        closed_pct: format!("{:.2}", closed_pct),
                                        open_count: term_open.len(),
                                        open_pct: format!("{:.2}", open_pct),
                                        completed_count: term_completed.len(),
                                        completed_pct: format!("{:.2}", completed_pct),
                                        wont_fix_count: term_wont_fix.len(),
                                        wont_fix_pct: format!("{:.2}", wont_fix_pct),
                                        duplicate_count: term_duplicate.len(),
                                        duplicate_pct: format!("{:.2}", duplicate_pct),
                                        bar_width_pct: format!("{:.2}", bar_width_pct),
                                        has_completed: completed_pct > 0.0,
                                        has_open: open_pct > 0.0,
                                        has_wont_fix: wont_fix_pct > 0.0,
                                        has_duplicate: duplicate_pct > 0.0,
                                    });
                                }
                                if !value_breakdowns.is_empty() {
                                    permutations.push(PermutationKeyBreakdown {
                                        key: key.clone(),
                                        values: value_breakdowns,
                                    });
                                }
                            }
                        }

                        let mut pairwise_matrices = vec![];
                        if !permutation_keys_values.is_empty() {
                            let keys: Vec<String> =
                                permutation_keys_values.keys().cloned().collect();
                            for i in 0..keys.len() {
                                for j in (i + 1)..keys.len() {
                                    let key_a = &keys[i];
                                    let key_b = &keys[j];

                                    let values_a: Vec<String> = permutation_keys_values
                                        .get(key_a)
                                        .unwrap()
                                        .iter()
                                        .cloned()
                                        .collect();
                                    let values_b: Vec<String> = permutation_keys_values
                                        .get(key_b)
                                        .unwrap()
                                        .iter()
                                        .cloned()
                                        .collect();

                                    // Calculate row total issues sum
                                    let sum_row_issues: usize = values_a
                                        .iter()
                                        .map(|val_a| {
                                            let label_a = crate::helpers::sanitize_label(format!(
                                                "{}: {}",
                                                key_a, val_a
                                            ));
                                            issues
                                                .iter()
                                                .filter(|i| {
                                                    i.labels.iter().any(|l| l.name == label_a)
                                                })
                                                .count()
                                        })
                                        .sum();

                                    // Calculate column total issues sum and individual column widths
                                    let sum_col_issues: usize = values_b
                                        .iter()
                                        .map(|val_b| {
                                            let label_b = crate::helpers::sanitize_label(format!(
                                                "{}: {}",
                                                key_b, val_b
                                            ));
                                            issues
                                                .iter()
                                                .filter(|i| {
                                                    i.labels.iter().any(|l| l.name == label_b)
                                                })
                                                .count()
                                        })
                                        .sum();

                                    let col_widths: Vec<f64> = values_b
                                        .iter()
                                        .map(|val_b| {
                                            let label_b = crate::helpers::sanitize_label(format!(
                                                "{}: {}",
                                                key_b, val_b
                                            ));
                                            let col_issues_cnt = issues
                                                .iter()
                                                .filter(|i| {
                                                    i.labels.iter().any(|l| l.name == label_b)
                                                })
                                                .count();
                                            if sum_col_issues > 0 {
                                                (col_issues_cnt as f64 / sum_col_issues as f64)
                                                    * 100.0
                                            } else {
                                                100.0 / (values_b.len() as f64)
                                            }
                                        })
                                        .collect();

                                    let headers = values_b
                                        .iter()
                                        .zip(&col_widths)
                                        .map(|(val_b, col_w)| MatrixHeader {
                                            value: val_b.clone(),
                                            width_pct: format!("{:.2}", col_w),
                                        })
                                        .collect();

                                    let mut rows = vec![];
                                    for val_a in &values_a {
                                        let label_a = crate::helpers::sanitize_label(format!(
                                            "{}: {}",
                                            key_a, val_a
                                        ));
                                        let row_issues_cnt = issues
                                            .iter()
                                            .filter(|i| i.labels.iter().any(|l| l.name == label_a))
                                            .count();
                                        let row_height = if sum_row_issues > 0 {
                                            50.0 + ((row_issues_cnt as f64 / sum_row_issues as f64)
                                                * 150.0)
                                        } else {
                                            50.0
                                        };

                                        let mut cells = vec![];
                                        for val_b in &values_b {
                                            let label_b = crate::helpers::sanitize_label(format!(
                                                "{}: {}",
                                                key_b, val_b
                                            ));

                                            let cell_issues = issues
                                                .iter()
                                                .filter(|issue| {
                                                    issue.labels.iter().any(|l| l.name == label_a)
                                                        && issue
                                                            .labels
                                                            .iter()
                                                            .any(|l| l.name == label_b)
                                                })
                                                .collect::<Vec<_>>();

                                            let cell_closed = cell_issues
                                                .iter()
                                                .filter(|issue| issue.state == IssueState::Closed)
                                                .collect::<Vec<_>>();
                                            let cell_open = cell_issues
                                                .iter()
                                                .filter(|issue| issue.state == IssueState::Open)
                                                .collect::<Vec<_>>();

                                            let cell_completed = cell_closed.iter().filter(|issue| {
                                                issue.state_reason != Some(octocrab::models::issues::IssueStateReason::NotPlanned)
                                                && !issue.labels.iter().any(|l| l.name.to_lowercase() == "duplicate")
                                            }).count();

                                            let cell_wont_fix = cell_closed.iter().filter(|issue| {
                                                issue.state_reason == Some(octocrab::models::issues::IssueStateReason::NotPlanned)
                                            }).count();

                                            let cell_duplicate = cell_closed
                                                .iter()
                                                .filter(|issue| {
                                                    issue.labels.iter().any(|l| {
                                                        l.name.to_lowercase() == "duplicate"
                                                    })
                                                })
                                                .count();

                                            let cell_total = cell_completed
                                                + cell_open.len()
                                                + cell_wont_fix
                                                + cell_duplicate;

                                            let cell_completed_pct = if cell_total > 0 {
                                                (cell_completed as f64 / cell_total as f64) * 100.0
                                            } else {
                                                0.0
                                            };
                                            let cell_open_pct = if cell_total > 0 {
                                                (cell_open.len() as f64 / cell_total as f64) * 100.0
                                            } else {
                                                0.0
                                            };
                                            let cell_wont_fix_pct = if cell_total > 0 {
                                                (cell_wont_fix as f64 / cell_total as f64) * 100.0
                                            } else {
                                                0.0
                                            };
                                            let cell_duplicate_pct = if cell_total > 0 {
                                                (cell_duplicate as f64 / cell_total as f64) * 100.0
                                            } else {
                                                0.0
                                            };

                                            let cell_completed_ratio = if cell_total > 0 {
                                                cell_completed as f64 / cell_total as f64
                                            } else {
                                                0.0
                                            };
                                            let cell_lightness =
                                                100.0 - (cell_completed_ratio * 12.0);

                                            cells.push(MatrixCell {
                                                total_cases: cell_issues.len(),
                                                completed: cell_completed,
                                                total_valid: cell_total,
                                                completed_pct: format!("{:.1}", cell_completed_pct),
                                                open: cell_open.len(),
                                                open_pct: format!("{:.1}", cell_open_pct),
                                                wont_fix: cell_wont_fix,
                                                wont_fix_pct: format!("{:.1}", cell_wont_fix_pct),
                                                duplicate: cell_duplicate,
                                                duplicate_pct: format!("{:.1}", cell_duplicate_pct),
                                                has_mini_bar: cell_total > 0,
                                                has_completed: cell_completed_pct > 0.0,
                                                has_open: cell_open_pct > 0.0,
                                                has_wont_fix: cell_wont_fix_pct > 0.0,
                                                has_duplicate: cell_duplicate_pct > 0.0,
                                                lightness: format!("{:.1}", cell_lightness),
                                            });
                                        }
                                        rows.push(MatrixRow {
                                            val_a: val_a.clone(),
                                            height: format!("{:.0}", row_height),
                                            cells,
                                        });
                                    }

                                    pairwise_matrices.push(PairwiseMatrix {
                                        key_a: key_a.clone(),
                                        key_b: key_b.clone(),
                                        headers,
                                        rows,
                                    });
                                }
                            }
                        }

                        let template = MetricsReportTemplate {
                            total_issues,
                            open_count,
                            open_pct,
                            completed_count,
                            completed_pct,
                            wont_fix_count,
                            wont_fix_pct,
                            duplicate_count,
                            duplicate_pct,
                            permutations,
                            pairwise_matrices,
                        };

                        let report_str = template.render().context("Failed to render the metrics report template")?;
                        std::fs::write(report_path, report_str)?;
                        println!("Report generated successfully to {:?}", report_path);
                    }                 }
                }
            }
            GithubSubcommands::Utils { subcommand } => {
                match subcommand {
                    GithubUtilsSubcommands::CloseIssues {
                        config_path,
                        label,
                        yes: is_confirmed,
                    } => {
                        let config = load_config(&config_path)?;
                        let gh = Github::new(&config.repo_owner, &config.repo_name)?;

                        if !is_confirmed {
                            let mut input = String::new();
                            println!("Are you sure you want to delete all of the existing labels? (yes/no)");
                            std::io::stdin()
                                .read_line(&mut input)
                                .context("Failed to read the user input")?;
                            if input.trim() != "yes" {
                                anyhow::bail!("The user did not confirm");
                            }
                        }

                        let issues = gh.get_issues(Some(octocrab::params::State::Open)).await?;
                        for issue in issues {
                            if issue.labels.iter().any(|gh_label| gh_label.name == label) {
                                println!("Closing issue: {}", &issue.title);
                                gh.close_issue(&issue).await?;
                            }
                        }
                    }
                    GithubUtilsSubcommands::ListLabels { config_path } => {
                        let config = load_config(&config_path)?;
                        let gh = Github::new(&config.repo_owner, &config.repo_name)?;

                        let labels = gh.get_labels().await?;
                        println!("Labels:");
                        for label in labels {
                            if let Some(description) = label.description {
                                println!("- {}: {}", label.name, description);
                            } else {
                                println!("- {}", label.name);
                            }
                        }
                    }
                    GithubUtilsSubcommands::DeleteAllLabels {
                        config_path,
                        yes: is_confirmed,
                    } => {
                        let config = load_config(&config_path)?;
                        let gh = Github::new(&config.repo_owner, &config.repo_name)?;

                        if !is_confirmed {
                            let mut input = String::new();
                            println!("Are you sure you want to delete all of the existing labels? (yes/no)");
                            std::io::stdin()
                                .read_line(&mut input)
                                .context("Failed to read the user input")?;
                            if input.trim() != "yes" {
                                anyhow::bail!("The user did not confirm");
                            }
                        }

                        gh.delete_labels().await?;
                        println!("Done 🚀");
                    }
                    GithubUtilsSubcommands::CreateLabels { config_path } => {
                        let config = load_config(&config_path)?;
                        let gh = Github::new(&config.repo_owner, &config.repo_name)?;

                        gh.create_labels(config.labels).await?;
                        println!("Done 🚀");
                    }
                }
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

    use crate::app::load_config::load_config;

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
    fn test_init() {
        let dir = tempdir().unwrap().path().to_path_buf();

        // run the init command
        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success()
            .stdout(predicate::str::contains("Created a YATM workspace in"));

        // load the config
        assert!(dir.join("config.yaml").is_file());
        let config = load_config(&dir).unwrap();

        // check that files are generated correctly
        assert!(config.new_requirements_dir.is_dir());
        assert!(config.new_test_cases_builder_dir.is_dir());
        assert!(config.generated_files_dir.is_dir());
        assert!(dir.join(".gitignore").is_file());
        assert_eq!(get_number_of_files_in_dir(&config.new_requirements_dir), 1);
    }

    #[test]
    fn test_requirements() {
        let dir = tempdir().unwrap().path().to_path_buf();

        // run the init command
        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success();

        // load the config
        assert!(dir.join("config.yaml").is_file());
        let config = load_config(&dir).unwrap();

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
        let new_requirements_file_path =
            config.new_requirements_dir.join(new_requirements_file_name);
        assert!(new_requirements_file_path.is_file());
        assert_eq!(get_number_of_files_in_dir(&config.new_requirements_dir), 2);

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

    #[test]
    fn test_test_cases() {
        let dir = tempdir().unwrap().path().to_path_buf();

        // run the init command
        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success();

        // load the config
        assert!(dir.join("config.yaml").is_file());
        let config = load_config(&dir).unwrap();

        // run the test cases new command
        let new_test_cases_builder_file_name = "my-test-test-cases-builder.yaml";
        let mut cmd = get_command();
        cmd.args(&[
            "test-cases",
            "new",
            "--config-path",
            dir.to_str().unwrap(),
            "--file-name",
            new_test_cases_builder_file_name,
        ])
        .assert()
        .success();
        let new_test_cases_builder_file_path = config
            .new_test_cases_builder_dir
            .join(new_test_cases_builder_file_name);
        assert!(new_test_cases_builder_file_path.is_file());
        assert_eq!(
            get_number_of_files_in_dir(&config.new_test_cases_builder_dir),
            2
        );

        // run the test cases list command
        let mut cmd = get_command();
        cmd.args(&["test-cases", "list", "--config-path", dir.to_str().unwrap()])
            .assert()
            .success();

        // run the test cases validate command
        let mut cmd = get_command();
        cmd.args(&[
            "test-cases",
            "validate",
            "--config-path",
            dir.to_str().unwrap(),
        ])
        .assert()
        .success();

        // fail on validating a bad directory
        let mut cmd = get_command();
        cmd.args(&[
            "test-cases",
            "validate",
            "--config-path",
            dir.join("bad").to_str().unwrap(),
        ])
        .assert()
        .failure();

        // run the test cases validate file command
        let mut cmd = get_command();
        cmd.args(&[
            "test-cases",
            "validate-file",
            "--file-path",
            new_test_cases_builder_file_path.to_str().unwrap(),
        ])
        .assert()
        .success();

        // run the test cases preview command
        assert_eq!(get_number_of_files_in_dir(&config.generated_files_dir), 0);
        let mut cmd = get_command();
        cmd.args(&[
            "test-cases",
            "preview",
            "--config-path",
            dir.to_str().unwrap(),
        ])
        .assert()
        .success();
        assert_eq!(get_number_of_files_in_dir(&config.generated_files_dir), 1);
    }

    #[test]
    #[cfg(feature = "github_tests")]
    fn test_github() {
        let dir = tempdir().unwrap().path().to_path_buf();

        // run the init command
        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success();

        // load the config
        assert!(dir.join("config.yaml").is_file());
        let config = load_config(&dir).unwrap();

        // run the github validate command
        let mut cmd = get_command();
        cmd.args(&["github", "validate", "--config-path", dir.to_str().unwrap()])
            .assert()
            .success();

        // fail on validating a bad directory
        let mut cmd = get_command();
        cmd.args(&[
            "github",
            "validate",
            "--config-path",
            dir.join("bad").to_str().unwrap(),
        ])
        .assert()
        .failure();

        // run the github preview command
        assert_eq!(get_number_of_files_in_dir(&config.generated_files_dir), 0);
        let mut cmd = get_command();
        cmd.args(&["github", "preview", "--config-path", dir.to_str().unwrap()])
            .assert()
            .success();
        assert_eq!(get_number_of_files_in_dir(&config.generated_files_dir), 1);

        // run the make label links command
        assert_eq!(get_number_of_files_in_dir(&config.generated_files_dir), 1);
        let mut cmd = get_command();
        cmd.args(&[
            "github",
            "make-label-links",
            "--config-path",
            dir.to_str().unwrap(),
        ])
        .assert()
        .success();
        assert_eq!(get_number_of_files_in_dir(&config.generated_files_dir), 2);
    }
}
