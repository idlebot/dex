use crate::platform::{self, Asset};
use reqwest::blocking::Client;
use serde::Deserialize;

// Only the fields we need from the GitHub API response.
#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Returns true if the URL looks like a GitHub repo or releases page.
pub fn is_github_release_url(url: &str) -> bool {
    parse_github_url(url).is_some()
}

/// Parses a GitHub URL into (owner, repo, optional tag).
///
/// Supported patterns:
///   https://github.com/owner/repo
///   https://github.com/owner/repo/releases
///   https://github.com/owner/repo/releases/latest
///   https://github.com/owner/repo/releases/tag/v1.2.3
fn parse_github_url(url: &str) -> Option<(&str, &str, Option<&str>)> {
    // Strip the scheme and domain prefix.
    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))?;

    let segments: Vec<&str> = path.trim_end_matches('/').split('/').collect();

    match segments.as_slice() {
        // https://github.com/owner/repo
        [owner, repo] => Some((owner, repo, None)),
        // https://github.com/owner/repo/releases or /releases/latest
        [owner, repo, "releases"] | [owner, repo, "releases", "latest"] => {
            Some((owner, repo, None))
        }
        // https://github.com/owner/repo/releases/tag/v1.2.3
        [owner, repo, "releases", "tag", tag] => Some((owner, repo, Some(tag))),
        _ => None,
    }
}

/// Resolves a GitHub release URL to a direct asset download URL.
///
/// Returns `(download_url, asset_filename)` for the best matching asset,
/// or an error if no suitable asset is found.
pub fn resolve_asset_url(
    url: &str,
    platform: &str,
    arch: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let (owner, repo, tag) = parse_github_url(url).ok_or("Not a valid GitHub release URL")?;

    let client = Client::builder()
        .user_agent(format!("dex/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let api_url = match tag {
        Some(t) => format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{t}"),
        None => format!("https://api.github.com/repos/{owner}/{repo}/releases/latest"),
    };

    let mut request = client.get(&api_url);

    // Use GITHUB_TOKEN for higher rate limits if available.
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("token {token}"));
    }

    let response = request.send()?.error_for_status()?;
    let release: GitHubRelease = serde_json::from_reader(response)?;

    // Convert GitHub assets into provider-agnostic Assets for matching.
    let assets: Vec<Asset> = release
        .assets
        .iter()
        .map(|a| Asset {
            name: a.name.clone(),
            url: a.browser_download_url.clone(),
        })
        .collect();

    let asset = platform::select_best_asset(&assets, platform, arch).ok_or_else(|| {
        format!(
            "No matching asset for platform={platform}, arch={arch} in release {}",
            release.tag_name
        )
    })?;

    eprintln!("Found: {repo} {} → {}", release.tag_name, asset.name);

    Ok((asset.url.clone(), asset.name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL parsing ─────────────────────────────────────────────────

    #[test]
    fn test_parse_owner_repo() {
        let result = parse_github_url("https://github.com/BurntSushi/ripgrep");
        assert_eq!(result, Some(("BurntSushi", "ripgrep", None)));
    }

    #[test]
    fn test_parse_releases() {
        let result = parse_github_url("https://github.com/BurntSushi/ripgrep/releases");
        assert_eq!(result, Some(("BurntSushi", "ripgrep", None)));
    }

    #[test]
    fn test_parse_releases_latest() {
        let result = parse_github_url("https://github.com/BurntSushi/ripgrep/releases/latest");
        assert_eq!(result, Some(("BurntSushi", "ripgrep", None)));
    }

    #[test]
    fn test_parse_releases_tag() {
        let result = parse_github_url("https://github.com/BurntSushi/ripgrep/releases/tag/14.1.1");
        assert_eq!(result, Some(("BurntSushi", "ripgrep", Some("14.1.1"))));
    }

    #[test]
    fn test_parse_trailing_slash() {
        let result = parse_github_url("https://github.com/BurntSushi/ripgrep/releases/");
        assert_eq!(result, Some(("BurntSushi", "ripgrep", None)));
    }

    #[test]
    fn test_parse_non_github_url() {
        assert!(parse_github_url("https://example.com/file.tar.gz").is_none());
    }

    #[test]
    fn test_parse_github_non_release_path() {
        assert!(parse_github_url("https://github.com/owner/repo/blob/main/file.rs").is_none());
    }

    #[test]
    fn test_parse_http_scheme() {
        let result = parse_github_url("http://github.com/owner/repo/releases");
        assert_eq!(result, Some(("owner", "repo", None)));
    }

    // ── is_github_release_url ───────────────────────────────────────

    #[test]
    fn test_is_github_release_url() {
        assert!(is_github_release_url("https://github.com/owner/repo"));
        assert!(is_github_release_url(
            "https://github.com/owner/repo/releases"
        ));
        assert!(is_github_release_url(
            "https://github.com/owner/repo/releases/tag/v1.0"
        ));
        assert!(!is_github_release_url("https://example.com/file.tar.gz"));
    }
}
