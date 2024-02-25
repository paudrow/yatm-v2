# README

YATM v2 is a tool for generating test cases from requirements for manual testing.
It works well with putting issues on Github so that community contributors can help with testing.

## Entry points

This repository contains a couple entry points.
You will find more information about each of them in their respective READMEs:

- [YATM v2](./src/yatm_v2/README.md)
- [Migration script for YATM v1 to v2](./src/migrate_v1_requirements/README.md)

## Setup

Make sure that you have [Rust](https://www.rust-lang.org/tools/install) installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Afterwards, you should be able to run each of the entry points.
You can either run the from the root of this repository with the `--bin` flag or by navigating to the respective directory and running `cargo run`.
For example:

```bash
# Run YATM v2 from the root of this repository
cargo run --bin yatm_v2

# Run YATM v2 from the yatm_v2 directory
# cd src/yatm_v2
cargo run
```

You can also install the entry points with `cargo install` and then you'll be able to access them from anywhere on your system.

```bash
# Install YATM v2 from the root of this repository
cargo install --path src/yatm_v2

# Install YATM v2 from the yatm_v2 directory
# cd src/yatm_v2
cargo install --path .
```

## Creating your own tools that work with YATM v2

If you want to add other tools that work with YATM v2, you can use the common types in `src/common`.

You can also write your own scripts to generate requirements from other sources and use YATM v2 to verify them.