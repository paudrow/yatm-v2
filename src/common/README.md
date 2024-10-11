# Common Module

The Common Module is a shared library for YATM v2 (Yet Another Test Manager) that provides common functionality and types used across different components of the project.

## Table of Contents

- [Overview](#overview)
- [Key Components](#key-components)
  - [GitHub Integration](#github-integration)
  - [Markdown Utilities](#markdown-utilities)
  - [Common Types](#common-types)
- [Usage](#usage)
- [Development](#development)
- [Testing](#testing)


## Overview

This module contains various utilities, types, and helpers that are used by other parts of the YATM v2 project. It includes functionality for working with GitHub, handling markdown, and defining common data structures.

## Key Components

### GitHub Integration

The module includes a `Github` struct that provides methods for interacting with the GitHub API. This is used for operations like creating issues, managing labels, and other GitHub-related tasks.

### Markdown Utilities

There's a `markdown_toc` module that provides functionality for generating and manipulating markdown tables of contents. It includes features like:

- Generating a table of contents from markdown content
- Customizing the table of contents output
- Prepending a table of contents to existing markdown content

For more details, see:

```
rust:src/common/src/markdown_toc.rs
startLine: 1
endLine: 327
```

### Common Types

The module defines several common types used throughout the project:

- `GithubLabel`: Represents a GitHub label with name, color, and description.
- `Requirement`: Defines the structure of a requirement, including steps, actions, and expectations.
- `RequirementsFile`: Represents a file containing multiple requirements.
- `TestCase`: Represents a test case derived from requirements.
- `TestCasesBuilder`: Defines how test cases should be built from requirements.

These types are crucial for the core functionality of YATM v2.

## Usage

To use this module in other parts of the YATM v2 project, add it as a dependency in your `Cargo.toml`:

```
[dependencies]
common = { path = "../common" }
```

Then, you can import and use the types and functions from this module in your Rust code:

```rust
use common::github::Github;
use common::types::{Requirement, TestCase};
use common::markdown_toc::prepend_markdown_table_of_contents;

// Use the imported types and functions as needed
```

## Development

When adding new functionality to the common module, ensure that it's truly shared across multiple components of the project. Keep the module focused on providing general-purpose utilities and types that are used in multiple places.

## Testing

The common module includes unit tests for its functionality. Run the tests using:

```bash
cargo test -p common
```

Ensure that any new functionality added to this module is appropriately tested.