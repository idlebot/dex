use std::fs;
use std::io;
use std::path::Path;

// An enum (short for "enumeration") in Rust is more powerful than in most languages.
// Each variant can hold data. Think of it as a "tagged union" — it's one of these
// types, and the compiler forces you to handle all cases.
// Unlike C/Java enums which are just named integers, Rust enums are full algebraic
// data types (if that means anything to you — if not, just think "fancy enum").
enum ArchiveType {
    TarGz,
    TarBz2,
    TarXz,
    TarZst,
    Zip,
    Gz,
    Bz2,
    Xz,
    Zst,
}

/// Checks if a file path has a recognized archive extension.
pub fn is_extractable(path: &Path) -> bool {
    detect_archive_type(path).is_some()
    // ↑ `is_some()` returns true if the Option is Some (not None).
}

/// Detects the archive type from the file extension.
/// Returns None if the extension isn't recognized.
fn detect_archive_type(path: &Path) -> Option<ArchiveType> {
    // No `pub` here — this is a private function, only visible within this module.

    // Convert the path to a lowercase string for case-insensitive matching.
    let path_str = path.to_string_lossy().to_lowercase();
    // ↑ `to_string_lossy()` converts a Path to a string, replacing any invalid
    //   Unicode characters with a replacement character. Paths on some OSes can
    //   contain non-UTF-8 bytes, so this is the safe conversion.

    // Check compound extensions first (e.g., .tar.gz before .gz).
    // Order matters here — ".tar.gz" would also match ".gz" if we checked that first.
    if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
        Some(ArchiveType::TarGz)
    } else if path_str.ends_with(".tar.bz2") || path_str.ends_with(".tbz2") {
        Some(ArchiveType::TarBz2)
    } else if path_str.ends_with(".tar.xz") || path_str.ends_with(".txz") {
        Some(ArchiveType::TarXz)
    } else if path_str.ends_with(".tar.zst") || path_str.ends_with(".tzst") {
        Some(ArchiveType::TarZst)
    } else if path_str.ends_with(".zip") {
        Some(ArchiveType::Zip)
    } else if path_str.ends_with(".gz") {
        Some(ArchiveType::Gz)
    } else if path_str.ends_with(".bz2") {
        Some(ArchiveType::Bz2)
    } else if path_str.ends_with(".xz") {
        Some(ArchiveType::Xz)
    } else if path_str.ends_with(".zst") {
        Some(ArchiveType::Zst)
    } else {
        None
    }
}

/// Extracts an archive file into the given output directory.
///
/// # Errors
/// Returns an error if the archive can't be read or extracted.
pub fn extract_file(path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // ↑ `Result<(), ...>` — the () is Rust's "unit type" (like void). It means
    //   "on success, there's no meaningful return value."

    fs::create_dir_all(output_dir)?;

    // Detect the archive type; if unknown, return an error.
    let archive_type = detect_archive_type(path).ok_or("Unknown archive format")?;
    // ↑ `ok_or` converts Option → Result: Some(v) → Ok(v), None → Err("message").
    //   Then `?` propagates the Err if it's None.

    match archive_type {
        ArchiveType::TarGz => extract_tar_gz(path, output_dir),
        ArchiveType::TarBz2 => extract_tar_bz2(path, output_dir),
        ArchiveType::TarXz => extract_tar_xz(path, output_dir),
        ArchiveType::TarZst => extract_tar_zst(path, output_dir),
        ArchiveType::Zip => extract_zip(path, output_dir),
        ArchiveType::Gz => extract_single_compressed(path, output_dir, "gz"),
        ArchiveType::Bz2 => extract_single_compressed(path, output_dir, "bz2"),
        ArchiveType::Xz => extract_single_compressed(path, output_dir, "xz"),
        ArchiveType::Zst => extract_single_compressed(path, output_dir, "zst"),
    }
}

// ========================================================================
// Private helper functions for each archive type.
// All follow the same pattern: open file → wrap in decompressor → extract.
// ========================================================================

/// Extracts a .tar.gz or .tgz archive.
fn extract_tar_gz(path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    // Wrap the file in a gzip decoder — it transparently decompresses as we read.
    let decoder = flate2::read::GzDecoder::new(file);
    // Wrap the decompressed stream in a tar archive reader.
    let mut archive = tar::Archive::new(decoder);
    // `unpack` extracts all files in the archive into the output directory.
    archive.unpack(output_dir)?;
    Ok(())
    // ↑ `Ok(())` = "success with no return value"
}

/// Extracts a .tar.bz2 or .tbz2 archive.
fn extract_tar_bz2(path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let decoder = bzip2::read::BzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(output_dir)?;
    Ok(())
}

/// Extracts a .tar.xz or .txz archive.
fn extract_tar_xz(path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let decoder = xz2::read::XzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(output_dir)?;
    Ok(())
}

/// Extracts a .tar.zst or .tzst archive.
fn extract_tar_zst(path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    // zstd::Decoder wraps a Read and decompresses on the fly.
    let decoder = zstd::Decoder::new(file)?;
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(output_dir)?;
    Ok(())
}

/// Extracts a .zip archive.
fn extract_zip(path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Zip files need to be extracted entry by entry (unlike tar which has `unpack`).
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        // `enclosed_name()` returns None if the entry path tries to escape the
        // output directory (e.g., "../../../etc/passwd"). This is a security measure
        // against "zip slip" attacks.
        let Some(entry_path) = entry.enclosed_name() else {
            // ↑ `let ... else` is Rust's way of doing "if this pattern doesn't match,
            //   execute the else block." The else block must diverge (return, break, continue).
            continue; // Skip malicious entries
        };

        let full_path = output_dir.join(entry_path);

        if entry.is_dir() {
            fs::create_dir_all(&full_path)?;
        } else {
            // Create parent directories if they don't exist.
            if let Some(parent) = full_path.parent() {
                // ↑ `if let` is like `match` but for a single pattern.
                //   "If full_path.parent() is Some(parent), do this block."
                fs::create_dir_all(parent)?;
            }
            let mut output_file = fs::File::create(&full_path)?;
            io::copy(&mut entry, &mut output_file)?;
            // ↑ `io::copy` streams bytes from a reader to a writer.
            //   Efficient — doesn't load the whole file into memory.
        }
    }

    Ok(())
}

/// Extracts a single compressed file (not a tar archive).
/// For example, "data.csv.gz" → decompresses to "data.csv".
fn extract_single_compressed(
    path: &Path,
    output_dir: &Path,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Figure out the output filename by stripping the compression extension.
    // e.g., "data.csv.gz" → "data.csv"
    let stem = path
        .file_name() // "data.csv.gz"
        .and_then(|n| n.to_str()) // Convert OsStr → &str
        // ↑ `and_then` chains Option operations: if None at any point, stays None.
        .and_then(|n| n.strip_suffix(&format!(".{format}"))) // Remove ".gz"
        .unwrap_or("decompressed"); // Fallback name if anything fails

    let output_path = output_dir.join(stem);
    let file = fs::File::open(path)?;
    let mut output_file = fs::File::create(&output_path)?;

    // Each format needs its own decompressor. We use `match` to pick the right one,
    // then `io::copy` streams the decompressed data to the output file.
    //
    // The `&mut` in `io::copy(&mut decoder, ...)` is needed because reading
    // consumes data (modifies the decoder's internal state), so we need a
    // mutable reference.
    match format {
        "gz" => {
            let mut decoder = flate2::read::GzDecoder::new(file);
            io::copy(&mut decoder, &mut output_file)?;
        }
        "bz2" => {
            let mut decoder = bzip2::read::BzDecoder::new(file);
            io::copy(&mut decoder, &mut output_file)?;
        }
        "xz" => {
            let mut decoder = xz2::read::XzDecoder::new(file);
            io::copy(&mut decoder, &mut output_file)?;
        }
        "zst" => {
            let mut decoder = zstd::Decoder::new(file)?;
            io::copy(&mut decoder, &mut output_file)?;
        }
        // `_` is the wildcard pattern — matches anything not covered above.
        // `unreachable!()` panics with a message — it should genuinely never happen
        // because we only call this function with known format strings.
        _ => unreachable!("Unknown single compression format: {format}"),
    }

    Ok(())
}

// ========================================================================
// Tests module. `#[cfg(test)]` means this code is ONLY compiled when running
// `cargo test` — it's stripped from release builds entirely.
// ========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    // ↑ `super::*` imports everything from the parent module (extract.rs itself).
    //   This gives tests access to all functions, including private ones.

    #[test]
    // ↑ `#[test]` marks this function as a test case. `cargo test` finds and runs all of these.
    fn test_detect_tar_gz() {
        assert!(is_extractable(Path::new("file.tar.gz")));
        assert!(is_extractable(Path::new("file.tgz")));
        assert!(is_extractable(Path::new("FILE.TAR.GZ"))); // case insensitive
    }

    #[test]
    fn test_detect_tar_bz2() {
        assert!(is_extractable(Path::new("file.tar.bz2")));
        assert!(is_extractable(Path::new("file.tbz2")));
    }

    #[test]
    fn test_detect_tar_xz() {
        assert!(is_extractable(Path::new("file.tar.xz")));
        assert!(is_extractable(Path::new("file.txz")));
    }

    #[test]
    fn test_detect_tar_zst() {
        assert!(is_extractable(Path::new("file.tar.zst")));
        assert!(is_extractable(Path::new("file.tzst")));
    }

    #[test]
    fn test_detect_zip() {
        assert!(is_extractable(Path::new("file.zip")));
        assert!(is_extractable(Path::new("FILE.ZIP")));
    }

    #[test]
    fn test_detect_single_compressed() {
        assert!(is_extractable(Path::new("file.gz")));
        assert!(is_extractable(Path::new("file.bz2")));
        assert!(is_extractable(Path::new("file.xz")));
        assert!(is_extractable(Path::new("file.zst")));
    }

    #[test]
    fn test_not_extractable() {
        assert!(!is_extractable(Path::new("file.txt")));
        assert!(!is_extractable(Path::new("file.pdf")));
        assert!(!is_extractable(Path::new("file")));
    }
}
