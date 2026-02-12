/// A downloadable asset with a name and URL.
/// Provider-agnostic — GitHub, GitLab, or any other source can produce these.
pub struct Asset {
    pub name: String,
    pub url: String,
}

/// Normalizes a Rust platform constant to the user-facing name.
pub fn normalize_platform(os: &str) -> &str {
    match os {
        "macos" | "darwin" => "macos",
        "linux" => "linux",
        "windows" => "windows",
        other => other,
    }
}

/// Normalizes a Rust architecture constant to the user-facing name.
pub fn normalize_arch(arch: &str) -> &str {
    match arch {
        "aarch64" => "arm64",
        "x86_64" => "x86_64",
        other => other,
    }
}

// Extensions that indicate non-downloadable files (checksums, signatures, etc.)
const SKIP_EXTENSIONS: &[&str] = &[
    ".sha256", ".sha512", ".sig", ".asc", ".sbom", ".json", ".txt", ".md",
];

// Words in the filename that indicate source archives (not binaries).
const SKIP_KEYWORDS: &[&str] = &["source", "src"];

// Platform alias groups: (canonical, &[aliases])
const PLATFORM_ALIASES: &[(&str, &[&str])] = &[
    ("linux", &["linux"]),
    ("macos", &["macos", "darwin", "osx", "apple"]),
    (
        "windows",
        &["windows", "win64", "win32", "win", "msvc", "pc-windows"],
    ),
];

// Architecture alias groups: (canonical, &[aliases])
const ARCH_ALIASES: &[(&str, &[&str])] = &[
    ("x86_64", &["x86_64", "x86-64", "amd64", "x64"]),
    ("arm64", &["arm64", "aarch64"]),
];

/// Selects the best matching asset from a list for the given platform and arch.
///
/// Scoring: +10 platform match, +5 arch match, +2 preferred format, +1 any extractable.
/// Returns `None` if no asset matches both platform and arch.
pub fn select_best_asset<'a>(assets: &'a [Asset], platform: &str, arch: &str) -> Option<&'a Asset> {
    let mut best: Option<(&Asset, i32)> = None;

    for asset in assets {
        let name_lower = asset.name.to_lowercase();

        // Skip non-downloadable files.
        if SKIP_EXTENSIONS.iter().any(|ext| name_lower.ends_with(ext)) {
            continue;
        }

        // Skip source archives.
        if SKIP_KEYWORDS.iter().any(|kw| name_lower.contains(kw)) {
            continue;
        }

        let mut score = 0;

        // Platform matching (+10)
        let platform_match = matches_alias_group(&name_lower, platform, PLATFORM_ALIASES);
        if platform_match {
            score += 10;
        }

        // Architecture matching (+5)
        let arch_match = matches_alias_group(&name_lower, arch, ARCH_ALIASES);
        if arch_match {
            score += 5;
        }

        // Must match both platform and arch to be considered.
        if !platform_match || !arch_match {
            continue;
        }

        // Prefer archive formats (+2 for preferred, +1 for any extractable)
        let is_windows = platform_aliases_contain(platform, "windows");
        if is_windows {
            if name_lower.ends_with(".zip") {
                score += 2;
            } else if is_extractable_ext(&name_lower) {
                score += 1;
            }
        } else if name_lower.ends_with(".tar.gz") || name_lower.ends_with(".tgz") {
            score += 2;
        } else if is_extractable_ext(&name_lower) {
            score += 1;
        }

        match &best {
            Some((_, best_score)) if score <= *best_score => {}
            _ => best = Some((asset, score)),
        }
    }

    best.map(|(asset, _)| asset)
}

/// Checks if a filename matches any alias in the group for the given canonical name.
fn matches_alias_group(name_lower: &str, value: &str, groups: &[(&str, &[&str])]) -> bool {
    let value_lower = value.to_lowercase();

    // Find the alias group that contains our value.
    for &(canonical, aliases) in groups {
        let is_in_group = canonical == value_lower || aliases.iter().any(|a| *a == value_lower);

        if is_in_group {
            // Check if the filename contains any alias from this group.
            return aliases.iter().any(|alias| name_lower.contains(alias));
        }
    }

    // If the value doesn't belong to any known group, do a direct substring match.
    name_lower.contains(&value_lower)
}

/// Checks if a platform value belongs to the "windows" group.
fn platform_aliases_contain(platform: &str, target: &str) -> bool {
    let platform_lower = platform.to_lowercase();
    for &(canonical, aliases) in PLATFORM_ALIASES {
        if canonical == target {
            return canonical == platform_lower || aliases.iter().any(|a| *a == platform_lower);
        }
    }
    false
}

/// Checks if a filename has a recognized extractable archive extension.
fn is_extractable_ext(name: &str) -> bool {
    let extractable = [
        ".tar.gz", ".tgz", ".tar.bz2", ".tbz2", ".tar.xz", ".txz", ".tar.zst", ".tzst", ".zip",
        ".gz", ".bz2", ".xz", ".zst",
    ];
    extractable.iter().any(|ext| name.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Normalize helpers ───────────────────────────────────────────

    #[test]
    fn test_normalize_platform() {
        assert_eq!(normalize_platform("macos"), "macos");
        assert_eq!(normalize_platform("darwin"), "macos");
        assert_eq!(normalize_platform("linux"), "linux");
        assert_eq!(normalize_platform("windows"), "windows");
        assert_eq!(normalize_platform("freebsd"), "freebsd");
    }

    #[test]
    fn test_normalize_arch() {
        assert_eq!(normalize_arch("aarch64"), "arm64");
        assert_eq!(normalize_arch("x86_64"), "x86_64");
        assert_eq!(normalize_arch("riscv64"), "riscv64");
    }

    // ── Asset matching ──────────────────────────────────────────────

    fn make_assets(names: &[&str]) -> Vec<Asset> {
        names
            .iter()
            .map(|n| Asset {
                name: n.to_string(),
                url: format!("https://example.com/{n}"),
            })
            .collect()
    }

    #[test]
    fn test_ripgrep_linux_x86_64() {
        let assets = make_assets(&[
            "ripgrep-14.1.1-aarch64-unknown-linux-gnu.tar.gz",
            "ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz",
            "ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
            "ripgrep-14.1.1-aarch64-apple-darwin.tar.gz",
            "ripgrep-14.1.1-x86_64-apple-darwin.tar.gz",
            "ripgrep-14.1.1.sha256",
        ]);

        let result = select_best_asset(&assets, "linux", "x86_64");
        assert_eq!(
            result.unwrap().name,
            "ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz"
        );
    }

    #[test]
    fn test_ripgrep_macos_arm64() {
        let assets = make_assets(&[
            "ripgrep-14.1.1-aarch64-unknown-linux-gnu.tar.gz",
            "ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz",
            "ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
            "ripgrep-14.1.1-aarch64-apple-darwin.tar.gz",
            "ripgrep-14.1.1-x86_64-apple-darwin.tar.gz",
        ]);

        let result = select_best_asset(&assets, "macos", "arm64");
        assert_eq!(
            result.unwrap().name,
            "ripgrep-14.1.1-aarch64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn test_windows_prefers_zip() {
        let assets = make_assets(&[
            "tool-1.0-x86_64-pc-windows-msvc.tar.gz",
            "tool-1.0-x86_64-pc-windows-msvc.zip",
            "tool-1.0-x86_64-unknown-linux-musl.tar.gz",
        ]);

        let result = select_best_asset(&assets, "windows", "x86_64");
        assert_eq!(result.unwrap().name, "tool-1.0-x86_64-pc-windows-msvc.zip");
    }

    #[test]
    fn test_linux_prefers_tar_gz() {
        let assets = make_assets(&[
            "tool-1.0-x86_64-unknown-linux-musl.zip",
            "tool-1.0-x86_64-unknown-linux-musl.tar.gz",
        ]);

        let result = select_best_asset(&assets, "linux", "x86_64");
        assert_eq!(
            result.unwrap().name,
            "tool-1.0-x86_64-unknown-linux-musl.tar.gz"
        );
    }

    #[test]
    fn test_skips_checksum_files() {
        let assets = make_assets(&[
            "tool-1.0-linux-amd64.tar.gz",
            "tool-1.0-linux-amd64.tar.gz.sha256",
            "tool-1.0-linux-amd64.tar.gz.sig",
        ]);

        let result = select_best_asset(&assets, "linux", "x86_64");
        assert_eq!(result.unwrap().name, "tool-1.0-linux-amd64.tar.gz");
    }

    #[test]
    fn test_skips_source_archives() {
        let assets = make_assets(&[
            "source.tar.gz",
            "tool-1.0-source-code.tar.gz",
            "tool-1.0-linux-amd64.tar.gz",
        ]);

        let result = select_best_asset(&assets, "linux", "x86_64");
        assert_eq!(result.unwrap().name, "tool-1.0-linux-amd64.tar.gz");
    }

    #[test]
    fn test_amd64_alias_matches_x86_64() {
        let assets = make_assets(&["tool-1.0-linux-amd64.tar.gz", "tool-1.0-linux-arm64.tar.gz"]);

        let result = select_best_asset(&assets, "linux", "x86_64");
        assert_eq!(result.unwrap().name, "tool-1.0-linux-amd64.tar.gz");
    }

    #[test]
    fn test_darwin_alias_matches_macos() {
        let assets = make_assets(&[
            "tool-1.0-darwin-arm64.tar.gz",
            "tool-1.0-linux-arm64.tar.gz",
        ]);

        let result = select_best_asset(&assets, "macos", "arm64");
        assert_eq!(result.unwrap().name, "tool-1.0-darwin-arm64.tar.gz");
    }

    #[test]
    fn test_no_matching_asset() {
        let assets = make_assets(&["tool-1.0-linux-amd64.tar.gz", "tool-1.0-linux-arm64.tar.gz"]);

        let result = select_best_asset(&assets, "windows", "x86_64");
        assert!(result.is_none());
    }

    #[test]
    fn test_all_filtered_out() {
        let assets = make_assets(&["checksums.sha256", "checksums.txt", "source.tar.gz"]);

        let result = select_best_asset(&assets, "linux", "x86_64");
        assert!(result.is_none());
    }

    #[test]
    fn test_fd_macos_x86_64() {
        let assets = make_assets(&[
            "fd-v10.2.0-aarch64-apple-darwin.tar.gz",
            "fd-v10.2.0-aarch64-unknown-linux-gnu.tar.gz",
            "fd-v10.2.0-aarch64-unknown-linux-musl.tar.gz",
            "fd-v10.2.0-arm-unknown-linux-gnueabihf.tar.gz",
            "fd-v10.2.0-arm-unknown-linux-musleabihf.tar.gz",
            "fd-v10.2.0-i686-pc-windows-msvc.zip",
            "fd-v10.2.0-i686-unknown-linux-gnu.tar.gz",
            "fd-v10.2.0-i686-unknown-linux-musl.tar.gz",
            "fd-v10.2.0-x86_64-apple-darwin.tar.gz",
            "fd-v10.2.0-x86_64-pc-windows-msvc.zip",
            "fd-v10.2.0-x86_64-unknown-linux-gnu.tar.gz",
            "fd-v10.2.0-x86_64-unknown-linux-musl.tar.gz",
        ]);

        let result = select_best_asset(&assets, "macos", "x86_64");
        assert_eq!(
            result.unwrap().name,
            "fd-v10.2.0-x86_64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn test_gh_cli_linux_arm64() {
        let assets = make_assets(&[
            "gh_2.60.0_linux_amd64.tar.gz",
            "gh_2.60.0_linux_arm64.tar.gz",
            "gh_2.60.0_macOS_amd64.zip",
            "gh_2.60.0_macOS_arm64.zip",
            "gh_2.60.0_windows_amd64.zip",
            "gh_2.60.0_checksums.txt",
        ]);

        let result = select_best_asset(&assets, "linux", "arm64");
        assert_eq!(result.unwrap().name, "gh_2.60.0_linux_arm64.tar.gz");
    }

    #[test]
    fn test_terraform_linux_amd64() {
        let assets = make_assets(&[
            "terraform_1.9.0_darwin_amd64.zip",
            "terraform_1.9.0_darwin_arm64.zip",
            "terraform_1.9.0_linux_amd64.zip",
            "terraform_1.9.0_linux_arm64.zip",
            "terraform_1.9.0_windows_amd64.zip",
            "terraform_1.9.0_SHA256SUMS",
            "terraform_1.9.0_SHA256SUMS.sig",
        ]);

        let result = select_best_asset(&assets, "linux", "x86_64");
        assert_eq!(result.unwrap().name, "terraform_1.9.0_linux_amd64.zip");
    }

    #[test]
    fn test_case_insensitive_matching() {
        let assets = make_assets(&[
            "Tool-1.0-Linux-X86_64.tar.gz",
            "Tool-1.0-Darwin-ARM64.tar.gz",
        ]);

        let result = select_best_asset(&assets, "linux", "x86_64");
        assert_eq!(result.unwrap().name, "Tool-1.0-Linux-X86_64.tar.gz");
    }

    #[test]
    fn test_user_passes_aarch64_directly() {
        let assets = make_assets(&["tool-1.0-linux-amd64.tar.gz", "tool-1.0-linux-arm64.tar.gz"]);

        let result = select_best_asset(&assets, "linux", "aarch64");
        assert_eq!(result.unwrap().name, "tool-1.0-linux-arm64.tar.gz");
    }

    #[test]
    fn test_user_passes_darwin_directly() {
        let assets = make_assets(&[
            "tool-1.0-darwin-arm64.tar.gz",
            "tool-1.0-linux-arm64.tar.gz",
        ]);

        let result = select_best_asset(&assets, "darwin", "arm64");
        assert_eq!(result.unwrap().name, "tool-1.0-darwin-arm64.tar.gz");
    }

    #[test]
    fn test_osx_alias() {
        let assets = make_assets(&["tool-1.0-osx-x86_64.tar.gz", "tool-1.0-linux-x86_64.tar.gz"]);

        let result = select_best_asset(&assets, "macos", "x86_64");
        assert_eq!(result.unwrap().name, "tool-1.0-osx-x86_64.tar.gz");
    }

    #[test]
    fn test_win64_alias() {
        let assets = make_assets(&["tool-1.0-win64-x64.zip", "tool-1.0-linux-amd64.tar.gz"]);

        let result = select_best_asset(&assets, "windows", "x86_64");
        assert_eq!(result.unwrap().name, "tool-1.0-win64-x64.zip");
    }
}
