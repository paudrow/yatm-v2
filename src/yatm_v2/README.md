# YATM v2

YATM v2 is a tool for generating test cases from requirements for manual testing.
It works well with putting issues on Github so that community contributors can help with testing.

YATM v2 is a rewrite of the [original YATM tool](https://github.com/audrow/yatm), which was written in Typescript and used for several [ROS 2](https://docs.ros.org/) and [Gazebo](https://gazebosim.org/home) releases by [Open Robotics](https://www.openrobotics.org/) and [Intrinsic AI](https://intrinsic.ai/).

## Features

- Specify requirements in YAML files
- Specify a test case builder to generate test cases from requirements
- CLI interface
- Upload test cases to Github issues (don't worry, it won't make duplicates)
- Preview test cases locally in a markdown file before uploading to Github
- Init a new YATM v2 project from the command line that sets up the config, file structure, and comes with sensible defaults
- Migrate requirements from YATM v1 to v2
- Verification commands to check for valid requirements and test case builder files. This also makes it easier to write code in other languages than Rust to generate requirements.
- CLI tool for getting metrics on test cases on Github
- Several Github utility functions to make common operations easier
- Uses a tag to version of your YATM v2 workspace to make it easy to separate different iterations of your test cases, for uploading and analytics purposes.
- Extensible design so you can build requirements builders.
- An templated file for the Github issue template, which makes it much easier to customize the issue template for your project.

## How to use YATM v2

### Pre-requisites

1. Create a Github repository for your project and note the owner and name of the repository.

2. Get a personal access token from Github. This token should have read and write access to the following.

  * `Actions` (not 100% sure if this is necessary)
  * `Contents`
  * `Issues`
  * `Pull requests`

  Note that you can give the token permissions to only the repositories you want to use YATM v2 with.

### Getting started

1. Install YATM v2 from the root of this repository

    ```bash
    cargo install --path src/yatm_v2
    ```
2. Initalize a new YATM v2 workspace

    ```bash
    yatm_v2 init --path /path/to/your/workspace
    ```

    This will create a new YATM v2 workspace with a `config.yaml` file and a `requirements` directory.

3. In `config.yaml`, modify the `repo_owner` and `repo_name` to match your Github repository.

4. Make sure that your workspace has access to your personal access token through the `GITHUB_TOKEN` environment variable.

    YATM v2 will look in the `.env` file in the root of your workspace for the `GITHUB_TOKEN` environment variable. This file should look like this:

    ```bash
    # .env
    GITHUB_TOKEN="github_pat_**********************************************************************************"
    ```

5. Generate a preview of your test cases:

    ```bash
    yatm_v2 github preview
    ```

    This will generate a markdown file with your test cases in the `generated_files_dir` directory, which is set in `config.yaml` and is `generated_files` by default.

    If this looks good, you can upload your test cases to Github.

6. Upload your test cases to Github:

    ```bash
    yatm_v2 github upload
    ```

    This will create a new issue on your Github repository with the test cases.

If you've gotten this far, you've successfully set up YATM v2 for your project. You can now start adding requirements and generating test cases.