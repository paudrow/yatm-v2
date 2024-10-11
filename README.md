# YATM v2 (Yet Another Test Manager)

YATM v2 is a powerful tool for generating and managing test cases from requirements for manual testing. It's particularly useful for projects that rely on community contributors for testing, as it integrates seamlessly with GitHub issues.

## Table of Contents

- [Key Features](#key-features)
- [Project Structure](#project-structure)
- [Quick Start](#quick-start)
- [Creating your own tools that work with YATM v2](#creating-your-own-tools-that-work-with-yatm-v2)
- [Contributing](#contributing)
- [License](#license)

## Key Features

- Specify requirements in YAML files
- Generate test cases from requirements using a customizable test case builder
- CLI interface for easy management
- Upload test cases to GitHub issues (with duplicate prevention)
- Preview test cases locally in markdown before uploading
- Initialize new YATM v2 projects with sensible defaults
- Migrate requirements from YATM v1 to v2
- Validate requirements and test case builder files
- Get metrics on test cases from GitHub
- Extensible design for custom requirement builders
- Customizable GitHub issue template

## Project Structure

This repository contains multiple entry points:

- [YATM v2](./src/yatm_v2): The main YATM v2 tool
- [Migration script for YATM v1 to v2](./src/migrate_v1_requirements): A tool to migrate YATM v1 requirements to v2 format

## Quick Start

1. Install Rust
2. Clone the repository
3. Build the project: `cargo build`
4. Install YATM v2: `cargo install --path src/yatm_v2`
5. Initialize a new workspace: `yatm_v2 init --path /path/to/your/workspace`
6. Configure your workspace (edit `config.yaml`)
7. Generate test cases: `yatm_v2 github preview`
8. Upload test cases to GitHub: `yatm_v2 github upload`

For more detailed setup instructions and usage information, please refer to the README files in the individual workspace modules:

- [YATM v2 Setup Instructions](./src/yatm_v2/README.md)
- [Migration Script Setup Instructions](./src/migrate_v1_requirements/README.md)

## Creating your own tools that work with YATM v2

If you want to add other tools that work with YATM v2, you can use the common types in `src/common`.

You can also write your own scripts to generate requirements from other sources and use YATM v2 to verify them.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.