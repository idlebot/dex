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

# Download a regular file (no extraction)
dex https://example.com/file.txt
```

## License

[MIT](LICENSE)
