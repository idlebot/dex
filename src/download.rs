use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Downloads a file from `url` into the `output_dir` directory.
/// Returns the full path to the downloaded file.
///
/// # Errors
/// Returns an error if the HTTP request fails, the server returns a non-success
/// status, or we can't write the file to disk.
//
// In Rust, `Result<T, E>` is the standard way to represent "this function can
// succeed with a value of type T, or fail with an error of type E."
// `Box<dyn std::error::Error>` means "any error type" — it's a trait object
// (like an interface pointer in OOP). This is the simple/lazy way to handle
// errors. Production code often defines custom error types instead.
pub fn download_file(url: &str, output_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // `pub` makes this function visible outside this module (from main.rs).
    // `&str` is a string reference (borrowed, read-only). `&Path` is similar but for paths.
    // The `&` means we're borrowing — we can read it but don't own it.
    // This is Rust's ownership system: only one owner at a time, others can borrow.

    // Build an HTTP client with a User-Agent header.
    // `?` at the end is the "try operator" — if the expression returns Err, immediately
    // return that error from our function. It's shorthand for:
    //   match result {
    //     Ok(val) => val,
    //     Err(e) => return Err(e.into()),
    //   }
    let client = Client::builder()
        .user_agent(format!("dex/{}", env!("CARGO_PKG_VERSION")))
        // ↑ `env!("CARGO_PKG_VERSION")` is a compile-time macro that reads the version
        //   from Cargo.toml. The string "dex/0.1.0" is baked into the binary.
        .build()?;

    // Send the GET request.
    let response = client.get(url).send()?.error_for_status()?;
    // ↑ Chaining: send the request, then check the HTTP status code.
    //   `error_for_status()` converts 4xx/5xx responses into Err values.
    //   Each `?` propagates errors upward.

    // Try to figure out the filename from the URL (the last path segment).
    // e.g., "https://example.com/files/archive.tar.gz" → "archive.tar.gz"
    let filename = url
        .rsplit('/') // Split from the right on '/'
        .next() // Take the first segment (= last part of URL)
        .unwrap_or("download")
        .split('?') // Strip query params (e.g., from CDN redirect URLs)
        .next()
        .unwrap_or("download")
        .to_string();
    // ↑ `.to_string()` converts a `&str` (borrowed) into a `String` (owned).
    //   We need ownership because we're building a PathBuf with it below.

    let file_path = output_dir.join(&filename);
    // ↑ `Path::join` concatenates paths with the correct separator.
    //   e.g., Path("./").join("file.tar.gz") → "./file.tar.gz"

    // Create the output directory if it doesn't exist.
    fs::create_dir_all(output_dir)?;

    // Get the total file size from the Content-Length header (if the server provides it).
    let total_size = response.content_length();
    // ↑ Returns Option<u64> — Some(size) or None if the header is missing.

    // Set up the progress bar.
    let pb = match total_size {
        Some(size) => {
            // If we know the total size, show a proper progress bar with percentage.
            let pb = ProgressBar::new(size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    // ↑ Template string for the progress bar appearance:
                    //   {spinner}     = spinning animation
                    //   {bar:40}      = 40-char wide progress bar
                    //   {bytes}       = downloaded so far
                    //   {total_bytes} = total file size
                    //   {eta}         = estimated time remaining
                    //   .green/.cyan/.blue = colors
                    .expect("invalid progress bar template")
                    .progress_chars("=> "),
                // ↑ Characters used to draw the bar: filled, current, empty
            );
            pb
        }
        None => {
            // If no Content-Length, show a spinner with byte count (no percentage).
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {bytes} downloaded")
                    .expect("invalid progress bar template"),
            );
            pb
        }
    };

    // Read the response body in chunks and write to the file.
    // This streams the data instead of loading the entire file into memory.
    let mut file = fs::File::create(&file_path)?;
    // ↑ `mut` = mutable. We need to write to this file, so it must be mutable.
    //   In Rust, variables are immutable by default (a safety feature).

    let mut downloaded: u64 = 0;
    let mut reader = response;

    // We read in 8KB chunks — a good balance between memory usage and I/O efficiency.
    let mut buffer = [0u8; 8192];
    // ↑ `[0u8; 8192]` creates an array of 8192 bytes, all initialized to 0.
    //   `u8` is an unsigned 8-bit integer (= a byte).

    loop {
        // `io::Read::read` fills the buffer and returns how many bytes were read.
        // 0 bytes means we've reached the end of the response body.
        let bytes_read = io::Read::read(&mut reader, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        // Write exactly the bytes we read (not the whole buffer) to the file.
        file.write_all(&buffer[..bytes_read])?;
        // ↑ `&buffer[..bytes_read]` is a slice — a view into part of the array.
        //   `[..n]` means "from index 0 up to (not including) n".
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    pb.finish_and_clear();
    // ↑ Remove the progress bar from the terminal when done.

    Ok(file_path)
    // ↑ Return the path wrapped in Ok — the success variant of Result.
    //   Note: no `return` keyword and no semicolon. In Rust, the last expression
    //   in a function is its return value (like Ruby or Kotlin).
}
