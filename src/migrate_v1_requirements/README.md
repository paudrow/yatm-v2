# YATM v1 to v2 Migration Script

This script facilitates the migration of YATM (Yet Another Test Manager) v1 requirements to YATM v2 format.

## Table of Contents

- [Features](#features)
- [Prerequisites](#prerequisites)

## Features

- Converts YATM v1 requirement files to YATM v2 format
- Supports custom output file naming and directory selection
- Provides options for overwriting existing files

## Prerequisites

Before using this migration script, ensure you have Rust installed on your system. If you haven't installed Rust yet, you can do so by running:

1. Install Rust

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

1. Install the migration script

    ```bash
    cargo install --path src/migrate_v1_requirements
    ```

    > You can uninstall it with `cargo uninstall migrate_v1_requirements`.

1. Run the migration script with the `--help` flag to see the available options

    ```bash
    migrate_v1_requirements --help
    ```

From there, you can run the migration script on your requirements files.

If you want to run the migration script on all of the requirements files in a directory, you can use a for loop like this:

```bash
for file in path/to/requirements/dir/*; do
  migrate_v1_requirements "$file" -o .
done
```
