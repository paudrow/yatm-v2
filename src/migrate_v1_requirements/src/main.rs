use anyhow::{Context, Result};
mod migrate_v1_to_v2;
mod requirements_file_v1;

use crate::requirements_file_v1::RequirementsFileV1;
use migrate_v1_to_v2::convert_requirements_file_v1_to_v2;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Input file path
    #[clap(value_parser, required = true)]
    input: PathBuf,

    /// Optional output file path
    #[clap(short, long, value_parser)]
    output: Option<PathBuf>,

    /// Overwrite the output file if it already exists
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    force: bool,
}

fn append_v2_to_filename(path: PathBuf) -> PathBuf {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let new_stem = format!("{}_v2", stem);
    let mut new_path = path.clone();

    if let Some(extension) = path.extension() {
        new_path.set_file_name(new_stem);
        new_path.set_extension(extension);
    } else {
        new_path.set_file_name(new_stem);
    }

    new_path
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let contents = std::fs::read_to_string(&cli.input)
        .context(format!("Could not read file: {:?}", &cli.input))?;

    // Determine the output path
    // If the output flag is set and is a directory, copy the input file name in the new directory
    let output_path = match cli.output {
        Some(path) => {
            if path.is_dir() {
                path.join(cli.input.clone().file_name().context(format!(
                    "Could not get input name: {}",
                    cli.input.to_string_lossy()
                ))?)
            } else {
                path
            }
        }
        None => append_v2_to_filename(cli.input.clone()),
    };

    if output_path.exists() && !cli.force {
        anyhow::bail!("Output file already exists: {:?}", &output_path);
    }

    // Parse the v1 requirements file and convert it to v2
    let requirements: RequirementsFileV1 = serde_yaml::from_str(&contents)?;
    let requirements_file_v2 =
        convert_requirements_file_v1_to_v2(&requirements).expect("Converts the requirements");

    // Write the v2 requirements file to the output path
    let requirements_file_v2 =
        serde_yaml::to_string(&requirements_file_v2).expect("Serializes to YAML");
    std::fs::write(&output_path, requirements_file_v2).context(format!(
        "Could not write file: {:}",
        &output_path.to_string_lossy()
    ))?;

    println!(
        "Successfully migrated v1 requirements to v2: {:}",
        &output_path.to_string_lossy()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::predicate;
    use std::path::Path;
    use tempfile::tempdir;

    fn get_command() -> Command {
        Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
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
    fn test_main() {
        let v1_file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/test_data/core.yaml");

        // copy the file to the temp directory
        let temp_dir = tempdir().expect("Creates a temporary directory");
        let temp_v1_file_path = temp_dir.path().join("requirements.yaml");
        let temp_v2_file_path = temp_dir.path().join("requirements_v2.yaml");
        let temp_v2_file_path_custom_output = temp_dir.path().join("my_file_v2.yaml");
        std::fs::copy(&v1_file_path, &temp_v1_file_path).expect("Copies a file");

        // run the command without the output flag
        assert!(!temp_v2_file_path.exists());
        let mut cmd = get_command();
        cmd.args(&[&temp_v1_file_path.to_str().unwrap()])
            .assert()
            .success();
        assert!(temp_v2_file_path.exists());

        // running the command without force fails to overwrite the existing file
        let mut cmd = get_command();
        cmd.args(&[&temp_v1_file_path.to_str().unwrap()])
            .assert()
            .failure();

        // running the command with force succeeds
        let mut cmd = get_command();
        cmd.args(&[&temp_v1_file_path.to_str().unwrap(), "--force"])
            .assert()
            .success();

        // running the command with the output flag
        assert!(!temp_v2_file_path_custom_output.exists());
        let mut cmd = get_command();
        cmd.args(&[
            &temp_v1_file_path.to_str().unwrap(),
            "--output",
            &temp_v2_file_path_custom_output.to_str().unwrap(),
        ])
        .assert()
        .success();
        assert!(temp_v2_file_path_custom_output.exists());
    }
}
