use clap::Parser;

/// dex - download and extract
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// URL to download
    url: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match cli.url {
        Some(url) => println!("Would download: {url}"),
        None => println!("No URL provided. Use --help for usage."),
    }
}
