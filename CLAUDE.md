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
- `cargo audit` — check dependencies for known vulnerabilities (requires `cargo install cargo-audit`)

## Project Structure

- `src/main.rs` — entry point and CLI definition
- `src/download.rs` — HTTP downloading with progress bar
- `src/extract.rs` — archive detection and extraction logic
- `src/platform.rs` — platform/arch detection, normalization, and asset matching (provider-agnostic)
- `src/github.rs` — GitHub release URL parsing and API interaction
- `Cargo.toml` — project metadata and dependencies
- `Cargo.lock` — pinned dependency versions (committed for binaries)

## Conventions

- Run `cargo fmt` before committing
- All code must pass `cargo clippy` with no warnings
- All tests must pass before merging PRs
- All changes go through pull requests — never push directly to main
- **PR titles must start with `[major]`, `[minor]`, or `[patch]`** — this is enforced by CI and controls automatic version bumping
  - `[patch]` — bug fixes, minor tweaks, docs
  - `[minor]` — new features, new CLI flags
  - `[major]` — breaking changes
- PRs are squash-merged, so the PR title becomes the commit message on main
- The version in `Cargo.toml` is a placeholder — release builds derive the version from git tags automatically
