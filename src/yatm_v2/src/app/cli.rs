use crate::app::init_workspace::init_workspace;
use crate::app::load_config::load_config;
use crate::constants::YAML_EXTENSIONS;
use crate::helpers::{
    describe_issue_diff, get_files, get_local_issues_matches, get_requirements_from_file,
    get_test_cases, get_test_cases_builders_from_file, permutation_to_labels,
    project_version_to_label, test_case_to_markdown, validate_requirements_file,
    validate_requirements_files, validate_test_cases_builder_file, GithubIssueMatches,
    IssueMatchType, SOURCE_BRANCH_ENV,
};
use crate::types::{Config, LocalIssue};
use common::github::Github;
use common::markdown_toc::{prepend_markdown_table_of_contents, TocOptions};
use common::types::{Link, RequirementsFile, TestCasesBuilderFile};

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
use clap::{Parser, Subcommand};
use octocrab::models::IssueState;
use octocrab::params::State;

// Define the main application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct App {
    #[clap(subcommand)]
    pub command: Commands,

    /// Branch to use in requirement source links (default: $YATM_SOURCE_BRANCH,
    /// then $GITHUB_BASE_REF, then the remote's default branch origin/HEAD, then main)
    #[clap(long, global = true)]
    pub source_branch: Option<String>,
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

        /// Print a unified diff (title and body) for each existing issue that would be updated
        #[clap(short = 'd', long)]
        show_diff: bool,
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
    /// Create the target repository (if configured) and reconcile collaborator permissions
    SetupRepo {
        /// The path to the project
        #[clap(short, long, default_value = ".")]
        config_path: PathBuf,
        /// Do not push things remotely
        #[clap(short = 'n', long)]
        dry_run: bool,
    },
}

async fn ensure_repository_and_permissions(
    gh: &Github,
    config: &Config,
    dry_run: bool,
) -> Result<bool> {
    let Some(settings) = &config.repo_settings else {
        if !dry_run {
            return Ok(true);
        }
        if gh.get_repository_state().await?.is_none() {
            anyhow::bail!(
                "Repository {}/{} does not exist and cannot be created automatically because the config has no repo_settings (set repo_settings.create_if_missing: true)",
                config.repo_owner,
                config.repo_name
            );
        }
        return Ok(true);
    };

    let remote_state = gh.get_repository_state().await?;
    let Some(remote_state) = remote_state else {
        if !settings.create_if_missing {
            anyhow::bail!(
                "Repository {}/{} does not exist and repo_settings.create_if_missing is false",
                config.repo_owner,
                config.repo_name
            );
        }

        if dry_run {
            // Fail validation now if the real run would be unable to create the repository.
            gh.check_can_create_repository().await?;
            let visibility = if settings.private {
                "private"
            } else {
                "public"
            };
            println!(
                "Dry run: Repository {}/{} does not exist; verified GITHUB_TOKEN can create it; would create {} repository.",
                config.repo_owner, config.repo_name, visibility
            );
            if !config.labels.is_empty() {
                println!(
                    "Dry run: Would create {} configured labels in {}/{}.",
                    config.labels.len(),
                    config.repo_owner,
                    config.repo_name
                );
            }
            for collaborator in &settings.collaborators {
                println!(
                    "Dry run: Would grant '{}' '{}' permission on {}/{}.",
                    collaborator.username,
                    collaborator.permission,
                    config.repo_owner,
                    config.repo_name
                );
            }
            if settings.strict_collaborators {
                println!(
                    "Dry run: strict_collaborators is enabled for {}/{}.",
                    config.repo_owner, config.repo_name
                );
            }
            return Ok(false);
        }

        println!(
            "Repository {}/{} not found. Creating...",
            config.repo_owner, config.repo_name
        );
        gh.create_repository(settings).await?;
        if !config.labels.is_empty() {
            gh.create_labels(config.labels.clone()).await?;
        }
        for collaborator in &settings.collaborators {
            gh.add_collaborator(collaborator).await?;
        }
        return Ok(true);
    };

    if !settings.reapply_settings_after_creation {
        return Ok(true);
    }

    if !Github::settings_match(&remote_state, settings) {
        if dry_run {
            println!(
                "Dry run: Repository settings for {}/{} do not match config; would update settings.",
                config.repo_owner, config.repo_name
            );
        } else {
            gh.update_repository_settings(settings).await?;
        }
    }

    for collaborator in &settings.collaborators {
        if dry_run {
            println!(
                "Dry run: Would ensure '{}' has '{}' permission on {}/{}.",
                collaborator.username, collaborator.permission, config.repo_owner, config.repo_name
            );
        } else {
            gh.add_collaborator(collaborator).await?;
        }
    }

    if settings.strict_collaborators {
        let allowed: HashSet<String> = settings
            .collaborators
            .iter()
            .map(|c| c.username.to_ascii_lowercase())
            .collect();

        let direct_collaborators = gh.list_direct_collaborators().await?;
        for existing in direct_collaborators {
            if existing.login.eq_ignore_ascii_case(&config.repo_owner) {
                continue;
            }
            if !allowed.contains(&existing.login.to_ascii_lowercase()) {
                if dry_run {
                    println!(
                        "Dry run: Would remove unlisted collaborator '{}' from {}/{}.",
                        existing.login, config.repo_owner, config.repo_name
                    );
                } else {
                    gh.remove_collaborator(&existing.login).await?;
                }
            }
        }

        let pending_invitations = gh.list_pending_invitations().await?;
        for invitation in pending_invitations {
            if let Some(invitee) = invitation.invitee {
                if !allowed.contains(&invitee.login.to_ascii_lowercase()) {
                    if dry_run {
                        println!(
                            "Dry run: Would revoke pending invitation for '{}' on {}/{}.",
                            invitee.login, config.repo_owner, config.repo_name
                        );
                    } else {
                        gh.delete_invitation(invitation.id, &invitee.login).await?;
                    }
                }
            }
        }
    }

    Ok(true)
}

pub async fn cli() -> Result<()> {
    let app = App::parse();
    if let Some(branch) = &app.source_branch {
        // Read by helpers::get_git_info when building requirement source links.
        std::env::set_var(SOURCE_BRANCH_ENV, branch);
    }
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

                file_contents = gh.prepend_github_info_with_settings(
                    &file_contents,
                    config.repo_settings.as_ref(),
                );

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
                show_diff,
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

                // Ensure the target repository and permissions are configured, then fetch issues
                let gh = Github::new(&config.repo_owner, &config.repo_name)?;
                println!(
                    "Connecting to repository: {}/{}",
                    config.repo_owner, config.repo_name
                );
                let repo_exists = ensure_repository_and_permissions(&gh, &config, dry_run).await?;
                let github_issues = if repo_exists {
                    gh.get_issues(Some(State::All)).await.with_context(|| {
                        if config.repo_settings.is_none() {
                            format!(
                                "Could not read issues from {}/{}. If the repository does not exist, add repo_settings with create_if_missing: true so it can be created automatically.",
                                config.repo_owner, config.repo_name
                            )
                        } else {
                            format!(
                                "Could not read issues from {}/{}",
                                config.repo_owner, config.repo_name
                            )
                        }
                    })?
                } else {
                    Vec::new()
                };

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
                    let github_issue = m.github_issue.as_ref().unwrap();
                    println!(
                        "Updating issue: {}, {}",
                        m.local_issue.title, github_issue.html_url
                    );
                    if show_diff {
                        let diff = describe_issue_diff(
                            &github_issue.title,
                            github_issue.body.as_deref().unwrap_or(""),
                            &m.local_issue.title,
                            &m.local_issue.text_body,
                        );
                        println!(
                            "Diff for issue #{} (- github, + local):\n{}",
                            github_issue.number,
                            diff.trim_end()
                        );
                    }
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
            GithubSubcommands::Metrics { config_path, label } => {
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

                    println!(
                        "{}/{} issues closed: {:.2}%",
                        closed_issues.len(),
                        issues.len(),
                        (closed_issues.len() as f64 / issues.len() as f64) * 100.0
                    );
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
                    GithubUtilsSubcommands::SetupRepo {
                        config_path,
                        dry_run,
                    } => {
                        let config = load_config(&config_path)?;
                        let gh = Github::new(&config.repo_owner, &config.repo_name)?;

                        ensure_repository_and_permissions(&gh, &config, dry_run).await?;
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
    fn test_upload_help_lists_show_diff() {
        let mut cmd = get_command();
        cmd.args(&["github", "upload", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--show-diff"))
            .stdout(predicate::str::contains("--dry-run"))
            .stdout(predicate::str::contains("--source-branch"));
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

    fn read_raw_http_request(stream: &mut std::net::TcpStream) -> String {
        use std::io::Read;
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = stream.read(&mut chunk).unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if let Some(header_end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&buf[..header_end]);
                let content_len = headers
                    .lines()
                    .find_map(|line| {
                        let lower = line.to_ascii_lowercase();
                        lower
                            .strip_prefix("content-length:")
                            .and_then(|v| v.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                if buf.len() >= header_end + 4 + content_len {
                    break;
                }
            }
        }
        String::from_utf8_lossy(&buf).to_string()
    }

    fn write_raw_http_response(stream: &mut std::net::TcpStream, status: &str, body: &str) {
        use std::io::Write;
        let resp = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            body.len(),
            body
        );
        stream.write_all(resp.as_bytes()).unwrap();
    }

    #[test]
    fn test_preview_and_dry_run_when_repo_missing() {
        let dir = tempdir().unwrap().path().to_path_buf();

        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success();

        // Update config with repo_settings and collaborators
        let config_path = dir.join("config.yaml");
        let mut config_str = std::fs::read_to_string(&config_path).unwrap();
        config_str = config_str
            .replace("repo_owner: repo_owner", "repo_owner: intrinsic-checklists")
            .replace("repo_name: repo_name", "repo_name: newhire-onboarding");
        config_str.push_str(
            "\nrepo_settings:\n  create_if_missing: true\n  private: true\n  description: Onboarding checklist\n  strict_collaborators: true\n  collaborators:\n  - username: mgr-user\n    permission: admin\n",
        );
        std::fs::write(&config_path, config_str).unwrap();

        // 1. Preview should succeed completely offline without GITHUB_TOKEN and render settings
        let mut cmd = get_command();
        cmd.env_remove("GITHUB_TOKEN")
            .args(&["github", "preview", "--config-path", dir.to_str().unwrap()])
            .assert()
            .success();

        let config = load_config(&dir).unwrap();
        let preview_file = std::fs::read_dir(&config.generated_files_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let preview_md = std::fs::read_to_string(preview_file).unwrap();
        assert!(preview_md.contains(
            "repo_settings: create_if_missing=true, reapply_settings_after_creation=true, visibility=private"
        ));
        assert!(preview_md.contains("strict_collaborators: true"));
        assert!(preview_md.contains("- `mgr-user` (`admin`)"));

        // 2. Upload --dry-run should succeed when target repo returns 404 Not Found,
        //    as long as the token is allowed to create it (probe returns 422).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let req = read_raw_http_request(&mut stream);
            assert!(req.starts_with("GET /repos/intrinsic-checklists/newhire-onboarding "));
            write_raw_http_response(&mut stream, "404 Not Found", r#"{"message":"Not Found"}"#);

            // Permission probe: nameless create request must never create a repo.
            let (mut stream, _) = listener.accept().unwrap();
            let req = read_raw_http_request(&mut stream);
            assert!(req.starts_with("POST /orgs/intrinsic-checklists/repos "));
            assert!(
                !req.contains("\"name\""),
                "probe must not send a name: {}",
                req
            );
            write_raw_http_response(
                &mut stream,
                "422 Unprocessable Entity",
                r#"{"message":"Repository creation failed.","errors":[{"resource":"Repository","code":"missing_field","field":"name"}]}"#,
            );
        });

        let mut cmd = get_command();
        cmd.env("GITHUB_TOKEN", "fake-token")
            .env("GITHUB_API_URL", format!("http://{}", addr))
            .args(&[
                "github",
                "upload",
                "--config-path",
                dir.to_str().unwrap(),
                "--dry-run",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Dry run: Repository intrinsic-checklists/newhire-onboarding does not exist; verified GITHUB_TOKEN can create it; would create private repository.",
            ))
            .stdout(predicate::str::contains(
                "Dry run: Would grant 'mgr-user' 'admin' permission on intrinsic-checklists/newhire-onboarding.",
            ))
            .stdout(predicate::str::contains("Done 🚀"));

        server.join().unwrap();

        // 3. Upload --dry-run must FAIL if the token is not allowed to create the repository.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let req = read_raw_http_request(&mut stream);
            assert!(req.starts_with("GET /repos/intrinsic-checklists/newhire-onboarding "));
            write_raw_http_response(&mut stream, "404 Not Found", r#"{"message":"Not Found"}"#);

            let (mut stream, _) = listener.accept().unwrap();
            let req = read_raw_http_request(&mut stream);
            assert!(req.starts_with("POST /orgs/intrinsic-checklists/repos "));
            use std::io::Write;
            let body = r#"{"message":"Resource not accessible by integration"}"#;
            let resp = format!(
                "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nX-Accepted-GitHub-Permissions: administration=write\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).unwrap();
        });

        let mut cmd = get_command();
        cmd.env("GITHUB_TOKEN", "fake-token")
            .env("GITHUB_API_URL", format!("http://{}", addr))
            .args(&[
                "github",
                "upload",
                "--config-path",
                dir.to_str().unwrap(),
                "--dry-run",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "GITHUB_TOKEN is not allowed to create repositories in 'intrinsic-checklists'",
            ))
            .stderr(predicate::str::contains("administration=write"));

        server.join().unwrap();
    }

    #[test]
    fn test_dry_run_fails_for_missing_repo_without_repo_settings() {
        let dir = tempdir().unwrap().path().to_path_buf();

        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success();

        let config_path = dir.join("config.yaml");
        let config_str = std::fs::read_to_string(&config_path)
            .unwrap()
            .replace("repo_owner: repo_owner", "repo_owner: intrinsic-checklists")
            .replace("repo_name: repo_name", "repo_name: legacy-onboarding");
        std::fs::write(&config_path, config_str).unwrap();

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let req = read_raw_http_request(&mut stream);
            assert!(req.starts_with("GET /repos/intrinsic-checklists/legacy-onboarding "));
            write_raw_http_response(&mut stream, "404 Not Found", r#"{"message":"Not Found"}"#);
        });

        let mut cmd = get_command();
        cmd.env("GITHUB_TOKEN", "fake-token")
            .env("GITHUB_API_URL", format!("http://{}", addr))
            .args(&[
                "github",
                "upload",
                "--config-path",
                dir.to_str().unwrap(),
                "--dry-run",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Repository intrinsic-checklists/legacy-onboarding does not exist and cannot be created automatically",
            ));

        server.join().unwrap();
    }

    #[test]
    fn test_setup_repo_creates_missing_repo_and_grants_collaborator() {
        let dir = tempdir().unwrap().path().to_path_buf();

        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success();

        let config_path = dir.join("config.yaml");
        let config_yaml = r#"repo_owner: intrinsic-checklists
repo_name: newhire-onboarding
repo_settings:
  create_if_missing: true
  private: true
  description: Onboarding checklist
  collaborators:
  - username: mgr-user
    permission: admin
workspace_version: 0.0.1
yatm_v2_version: 1.0.0
requirements_dirs:
- requirements
test_cases_builders_dirs:
- test_cases_builders
new_requirements_dir: requirements
new_test_cases_builder_dir: test_cases_builders
generated_files_dir: generated_files
labels: []
"#;
        std::fs::write(&config_path, config_yaml).unwrap();

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            // 1. GET /repos/intrinsic-checklists/newhire-onboarding -> 404
            let (mut s1, _) = listener.accept().unwrap();
            let req1 = read_raw_http_request(&mut s1);
            assert!(req1.starts_with("GET /repos/intrinsic-checklists/newhire-onboarding "));
            write_raw_http_response(&mut s1, "404 Not Found", r#"{"message":"Not Found"}"#);

            // 2. POST /orgs/intrinsic-checklists/repos -> 201
            let (mut s2, _) = listener.accept().unwrap();
            let req2 = read_raw_http_request(&mut s2);
            assert!(req2.starts_with("POST /orgs/intrinsic-checklists/repos "));
            assert!(req2.contains(r#""name":"newhire-onboarding""#));
            write_raw_http_response(&mut s2, "201 Created", r#"{"id":1}"#);

            // 3. GET /repos/intrinsic-checklists/newhire-onboarding -> 200
            let (mut s3, _) = listener.accept().unwrap();
            let req3 = read_raw_http_request(&mut s3);
            assert!(req3.starts_with("GET /repos/intrinsic-checklists/newhire-onboarding "));
            write_raw_http_response(&mut s3, "200 OK", r#"{"id":1}"#);

            // 4. PUT /repos/intrinsic-checklists/newhire-onboarding/collaborators/mgr-user -> 201
            let (mut s4, _) = listener.accept().unwrap();
            let req4 = read_raw_http_request(&mut s4);
            assert!(req4.starts_with(
                "PUT /repos/intrinsic-checklists/newhire-onboarding/collaborators/mgr-user "
            ));
            write_raw_http_response(&mut s4, "201 Created", r#"{"id":42}"#);
        });

        let mut cmd = get_command();
        cmd.env("GITHUB_TOKEN", "fake-token")
            .env("GITHUB_API_URL", format!("http://{}", addr))
            .args(&[
                "github",
                "utils",
                "setup-repo",
                "--config-path",
                dir.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Created private repository intrinsic-checklists/newhire-onboarding",
            ))
            .stdout(predicate::str::contains(
                "Invited 'mgr-user' with 'admin' permission to intrinsic-checklists/newhire-onboarding",
            ))
            .stdout(predicate::str::contains("Done 🚀"));

        server.join().unwrap();
    }

    #[test]
    fn test_existing_repo_without_repo_settings_makes_no_admin_changes() {
        let dir = tempdir().unwrap().path().to_path_buf();

        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success();

        // Legacy config: no repo_settings and no collaborators.
        // Bind a listener that panics if ANY admin/repo HTTP call is made during setup-repo!
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let mut cmd = get_command();
        cmd.env("GITHUB_TOKEN", "fake-token")
            .env("GITHUB_API_URL", format!("http://{}", addr))
            .args(&[
                "github",
                "utils",
                "setup-repo",
                "--config-path",
                dir.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Done 🚀"));

        listener.set_nonblocking(true).unwrap();
        assert!(
            listener.accept().is_err(),
            "Expected zero administrative API calls when repo_settings and collaborators are unset"
        );
    }

    #[test]
    fn test_reapply_settings_after_creation_and_strict_collaborators() {
        let dir = tempdir().unwrap().path().to_path_buf();

        let mut cmd = get_command();
        cmd.args(&["init", "--path", dir.to_str().unwrap()])
            .assert()
            .success();

        let config_path = dir.join("config.yaml");
        let config_yaml = r#"repo_owner: intrinsic-checklists
repo_name: newhire-onboarding
repo_settings:
  create_if_missing: true
  private: true
  description: Onboarding checklist
  strict_collaborators: true
  collaborators:
  - username: Mgr-User
    permission: admin
workspace_version: 0.0.1
yatm_v2_version: 1.0.0
requirements_dirs:
- requirements
test_cases_builders_dirs:
- test_cases_builders
new_requirements_dir: requirements
new_test_cases_builder_dir: test_cases_builders
generated_files_dir: generated_files
labels: []
"#;
        std::fs::write(&config_path, config_yaml).unwrap();

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            // 1. GET /repos/intrinsic-checklists/newhire-onboarding -> mismatched settings
            let (mut s1, _) = listener.accept().unwrap();
            let req1 = read_raw_http_request(&mut s1);
            assert!(req1.starts_with("GET /repos/intrinsic-checklists/newhire-onboarding "));
            write_raw_http_response(
                &mut s1,
                "200 OK",
                r#"{"id":1,"private":false,"description":"Outdated description"}"#,
            );

            // 2. PATCH /repos/intrinsic-checklists/newhire-onboarding -> updates settings
            let (mut s2, _) = listener.accept().unwrap();
            let req2 = read_raw_http_request(&mut s2);
            assert!(req2.starts_with("PATCH /repos/intrinsic-checklists/newhire-onboarding "));
            assert!(req2.contains(r#""private":true"#));
            assert!(req2.contains(r#""description":"Onboarding checklist""#));
            write_raw_http_response(
                &mut s2,
                "200 OK",
                r#"{"id":1,"private":true,"description":"Onboarding checklist"}"#,
            );

            // 3. PUT /repos/intrinsic-checklists/newhire-onboarding/collaborators/Mgr-User
            let (mut s3, _) = listener.accept().unwrap();
            let req3 = read_raw_http_request(&mut s3);
            assert!(req3.starts_with(
                "PUT /repos/intrinsic-checklists/newhire-onboarding/collaborators/Mgr-User "
            ));
            write_raw_http_response(&mut s3, "204 No Content", "");

            // 4. GET direct collaborators -> mgr-user (diff case) kept, unlisted-user removed
            let (mut s4, _) = listener.accept().unwrap();
            let req4 = read_raw_http_request(&mut s4);
            assert!(req4.starts_with(
                "GET /repos/intrinsic-checklists/newhire-onboarding/collaborators?affiliation=direct&per_page=100&page=1 "
            ));
            write_raw_http_response(
                &mut s4,
                "200 OK",
                r#"[{"login":"mgr-user"},{"login":"unlisted-user"}]"#,
            );

            // 5. DELETE unlisted-user
            let (mut s5, _) = listener.accept().unwrap();
            let req5 = read_raw_http_request(&mut s5);
            assert!(req5.starts_with(
                "DELETE /repos/intrinsic-checklists/newhire-onboarding/collaborators/unlisted-user "
            ));
            write_raw_http_response(&mut s5, "204 No Content", "");

            // 6. GET pending invitations -> MGR-USER kept, manual-invitee revoked
            let (mut s6, _) = listener.accept().unwrap();
            let req6 = read_raw_http_request(&mut s6);
            assert!(req6.starts_with(
                "GET /repos/intrinsic-checklists/newhire-onboarding/invitations?per_page=100&page=1 "
            ));
            write_raw_http_response(
                &mut s6,
                "200 OK",
                r#"[{"id":55,"invitee":{"login":"manual-invitee"}},{"id":56,"invitee":{"login":"MGR-USER"}}]"#,
            );

            // 7. DELETE invitation 55
            let (mut s7, _) = listener.accept().unwrap();
            let req7 = read_raw_http_request(&mut s7);
            assert!(req7.starts_with(
                "DELETE /repos/intrinsic-checklists/newhire-onboarding/invitations/55 "
            ));
            write_raw_http_response(&mut s7, "204 No Content", "");
        });

        let mut cmd = get_command();
        cmd.env("GITHUB_TOKEN", "fake-token")
            .env("GITHUB_API_URL", format!("http://{}", addr))
            .args(&[
                "github",
                "utils",
                "setup-repo",
                "--config-path",
                dir.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Updated repository settings for intrinsic-checklists/newhire-onboarding",
            ))
            .stdout(predicate::str::contains(
                "Removed unlisted collaborator 'unlisted-user' from intrinsic-checklists/newhire-onboarding",
            ))
            .stdout(predicate::str::contains(
                "Revoked pending invitation for 'manual-invitee' on intrinsic-checklists/newhire-onboarding",
            ))
            .stdout(predicate::str::contains("Done 🚀"));

        server.join().unwrap();

        // Now set reapply_settings_after_creation: false and verify neither settings nor collaborators
        // are updated once the repository already exists.
        let no_reapply_yaml = config_yaml.replace(
            "  create_if_missing: true",
            "  create_if_missing: true\n  reapply_settings_after_creation: false",
        );
        std::fs::write(&config_path, no_reapply_yaml).unwrap();

        let listener2 = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr2 = listener2.local_addr().unwrap();
        let server2 = std::thread::spawn(move || {
            let (mut s1, _) = listener2.accept().unwrap();
            let req1 = read_raw_http_request(&mut s1);
            assert!(req1.starts_with("GET /repos/intrinsic-checklists/newhire-onboarding "));
            write_raw_http_response(
                &mut s1,
                "200 OK",
                r#"{"id":1,"private":false,"description":"Outdated description"}"#,
            );

            listener2.set_nonblocking(true).unwrap();
            assert!(
                listener2.accept().is_err(),
                "Expected no PATCH/PUT/DELETE calls when reapply_settings_after_creation is false"
            );
        });

        let mut cmd = get_command();
        cmd.env("GITHUB_TOKEN", "fake-token")
            .env("GITHUB_API_URL", format!("http://{}", addr2))
            .args(&[
                "github",
                "utils",
                "setup-repo",
                "--config-path",
                dir.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Done 🚀"));

        server2.join().unwrap();
    }
}
