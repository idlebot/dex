# dex

A command-line tool that downloads files and automatically extracts archives.

Think `curl` or `wget`, but if the file is an archive, it gets extracted for you.

## Supported Formats

- `.tar.gz` / `.tgz`
- `.tar.bz2` / `.tbz2`
- `.tar.xz` / `.txz`
- `.tar.zst` / `.tzst`
- `.zip`
- `.gz`
- `.bz2`
- `.xz`
- `.zst`

## Installation

### From GitHub Releases

Download a prebuilt binary from the [releases page](https://github.com/idlebot/dex/releases).

### From Source

```bash
cargo install --git https://github.com/idlebot/dex
```

## Usage

```bash
# Download and auto-extract an archive
dex https://example.com/archive.tar.gz

# Extract to a specific directory
dex https://example.com/archive.tar.gz -o ./mydir

# Download without extracting
dex https://example.com/archive.tar.gz --no-extract

# Download, extract, and keep the original archive
dex https://example.com/archive.tar.gz --keep

# Download a regular file (no extraction needed)
dex https://example.com/file.txt
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

[MIT](LICENSE)
