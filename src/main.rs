// `mod` declares a module — tells Rust "there's a file called download.rs (or extract.rs)
// in the same directory, include it as part of this program."
// This is how Rust splits code across files. Each `mod` = one file.
mod download;
mod extract;

use clap::Parser;
use std::path::PathBuf;

/// dex - download and extract
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// URL to download
    url: String,
    // ↑ Changed from Option<String> to String — now the URL is required.
    //   clap will show an error if omitted.
    /// Output directory for extracted files (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    // ↑ `#[arg(...)]` customizes how clap handles this field:
    //   `short` = accept `-o`, `long` = accept `--output`
    //   `default_value` = use "." if not provided
    output: PathBuf,
    // ↑ PathBuf is Rust's owned file path type (like String but for paths).
    //   It handles OS-specific path separators (/ vs \) correctly.
    /// Keep the downloaded archive (don't delete after extracting)
    #[arg(short, long)]
    keep: bool,
    // ↑ Boolean flags are false by default. Passing `--keep` or `-k` sets it to true.
    /// Don't extract, just download
    #[arg(short, long)]
    no_extract: bool,
}

fn main() {
    let cli = Cli::parse();

    // Download the file. If this fails, print the error and exit.
    // `unwrap_or_else` is a common Rust pattern: "use the Ok value, or if it's
    // an Err, run this closure (anonymous function) instead."
    // `|e|` is closure syntax — `e` is the error value.
    // `eprintln!` prints to stderr (not stdout), which is convention for errors.
    // `std::process::exit(1)` exits with a non-zero code (= failure).
    let downloaded_path = download::download_file(&cli.url, &cli.output).unwrap_or_else(|e| {
        eprintln!("Error downloading: {e}");
        std::process::exit(1);
    });

    // If --no-extract was passed, we're done.
    if cli.no_extract {
        println!("Saved to {}", downloaded_path.display());
        return;
        // ↑ `return` exits the function early, like in most languages.
    }

    // Check if the file is an extractable archive based on its extension.
    if extract::is_extractable(&downloaded_path) {
        extract::extract_file(&downloaded_path, &cli.output).unwrap_or_else(|e| {
            eprintln!("Error extracting: {e}");
            std::process::exit(1);
        });

        // Delete the archive unless --keep was passed.
        if !cli.keep {
            // `let _ =` means "I don't care about the return value."
            // fs::remove_file could fail (e.g., permission denied) but if extraction
            // succeeded, failing to clean up the archive isn't fatal.
            let _ = std::fs::remove_file(&downloaded_path);
        }

        println!("Extracted to {}", cli.output.display());
    } else {
        println!("Saved to {}", downloaded_path.display());
    }
}
