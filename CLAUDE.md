# CLAUDE.md

## Project Overview

dex is a CLI download tool (like curl/wget) that automatically extracts archives.
Written in Rust, using clap for CLI argument parsing.

## Build Commands

- `cargo build` — debug build
- `cargo build --release` — optimized release build
- `cargo test` — run all tests
- `cargo clippy` — run the Rust linter
- `cargo fmt --check` — check code formatting

## Project Structure

- `src/main.rs` — entry point and CLI definition
- `Cargo.toml` — project metadata and dependencies
- `Cargo.lock` — pinned dependency versions (committed for binaries)

## Conventions

- Run `cargo fmt` before committing
- All code must pass `cargo clippy` with no warnings
- All tests must pass before merging PRs
- Commit messages should be prefixed with `[major]`, `[minor]`, or `[patch]` to trigger auto-versioning
