# README

This is a migration script for YATM v1 requirements to YATM v2 requirements.

## Getting started

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
