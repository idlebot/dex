mod download;
mod extract;
mod github;
mod platform;

use clap::Parser;
use std::path::PathBuf;

/// dex - download and extract
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// URL to download
    url: String,

    /// Output directory for extracted files (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    output: PathBuf,

    /// Keep the downloaded archive (don't delete after extracting)
    #[arg(short, long)]
    keep: bool,

    /// Don't extract, just download
    #[arg(short, long)]
    no_extract: bool,

    /// Override platform detection (e.g., linux, macos, windows)
    #[arg(long)]
    platform: Option<String>,

    /// Override architecture detection (e.g., x86_64, arm64)
    #[arg(long)]
    arch: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Determine the effective URL — resolve GitHub release URLs to direct asset URLs.
    let url = if github::is_github_release_url(&cli.url) {
        let platform = cli
            .platform
            .as_deref()
            .unwrap_or_else(|| platform::normalize_platform(std::env::consts::OS));
        let arch = cli
            .arch
            .as_deref()
            .unwrap_or_else(|| platform::normalize_arch(std::env::consts::ARCH));

        let (asset_url, _filename) = github::resolve_asset_url(&cli.url, platform, arch)
            .unwrap_or_else(|e| {
                eprintln!("Error resolving GitHub release: {e}");
                std::process::exit(1);
            });

        asset_url
    } else {
        cli.url.clone()
    };

    let downloaded_path = download::download_file(&url, &cli.output).unwrap_or_else(|e| {
        eprintln!("Error downloading: {e}");
        std::process::exit(1);
    });

    if cli.no_extract {
        println!("Saved to {}", downloaded_path.display());
        return;
    }

    if extract::is_extractable(&downloaded_path) {
        extract::extract_file(&downloaded_path, &cli.output).unwrap_or_else(|e| {
            eprintln!("Error extracting: {e}");
            std::process::exit(1);
        });

        if !cli.keep {
            let _ = std::fs::remove_file(&downloaded_path);
        }

        println!("Extracted to {}", cli.output.display());
    } else {
        println!("Saved to {}", downloaded_path.display());
    }
}
