# Contributing to dex

## Development Setup

1. Install [Rust](https://rustup.rs/)
2. Clone the repo:
   ```bash
   git clone https://github.com/idlebot/dex.git
   cd dex
   ```
3. Build and run:
   ```bash
   cargo build
   cargo run -- https://example.com/file.tar.gz
   ```

## Making Changes

All changes go through pull requests. Direct pushes to `main` are not allowed.

### PR Title Format (Required)

PR titles **must** start with a version prefix:

| Prefix | When to use | Example |
|--------|-------------|---------|
| `[patch]` | Bug fixes, docs, minor tweaks | `[patch] Fix timeout on large downloads` |
| `[minor]` | New features, new CLI flags | `[minor] Add --timeout flag` |
| `[major]` | Breaking changes | `[major] Change default output directory behavior` |

This is enforced by CI — PRs without a valid prefix cannot be merged. The prefix controls automatic version bumping: when your PR is squash-merged, the title becomes the commit message on `main`, and the auto-tag workflow reads it to determine the next release version.

### Workflow

1. Create a branch from `main`
2. Make your changes
3. Ensure all checks pass locally:
   ```bash
   cargo fmt        # format code
   cargo clippy     # lint
   cargo test       # run tests
   ```
4. Push and open a PR with a properly prefixed title
5. Wait for CI to pass (tests, clippy, formatting, title check)
6. PRs are squash-merged into `main`

## Project Structure

```
src/
  main.rs       # CLI definition and entry point
  download.rs   # HTTP downloading with progress bar
  extract.rs    # Archive detection and extraction
```

## Adding a New Archive Format

1. Add the decompression crate to `Cargo.toml`
2. Add the format to `ArchiveType` enum in `src/extract.rs`
3. Add detection logic in `detect_archive_type()`
4. Add an extraction function
5. Add tests for the new format
6. Update the supported formats list in `README.md`
