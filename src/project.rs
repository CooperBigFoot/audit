use std::path::{Path, PathBuf};

use tracing::debug;

use crate::error::ProjectError;
use crate::types::ProjectName;

/// Detect project name from the git repository at or above `working_dir`.
pub fn detect_project(working_dir: &Path) -> Result<ProjectName, ProjectError> {
    debug!(dir = %working_dir.display(), "detecting project");

    // Try git remote URL first
    if let Some(name) = detect_from_remote(working_dir) {
        debug!(project = %name, "detected from git remote");
        return Ok(name);
    }

    // Fall back to repo root directory name
    if let Some(name) = detect_from_toplevel(working_dir) {
        debug!(project = %name, "detected from repo toplevel");
        return Ok(name);
    }

    Err(ProjectError::CannotDetect {
        path: working_dir.to_path_buf(),
    })
}

/// Walk up from `start` looking for a `.git` directory or file.
fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let dot_git = current.join(".git");
        if dot_git.exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn detect_from_remote(dir: &Path) -> Option<ProjectName> {
    let root = find_git_root(dir)?;
    let git_config_path = root.join(".git").join("config");
    let content = std::fs::read_to_string(git_config_path).ok()?;
    let url = parse_origin_url(&content)?;
    parse_repo_name(&url)
}

fn detect_from_toplevel(dir: &Path) -> Option<ProjectName> {
    let root = find_git_root(dir)?;
    let basename = root.file_name()?.to_str()?;
    basename.parse().ok()
}

/// Extract the URL from the `[remote "origin"]` section of a git config.
fn parse_origin_url(config: &str) -> Option<String> {
    let mut in_origin = false;
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_origin = trimmed == r#"[remote "origin"]"#;
            continue;
        }
        if in_origin {
            if let Some(rest) = trimmed.strip_prefix("url") {
                let rest = rest.trim_start();
                if let Some(url) = rest.strip_prefix('=') {
                    return Some(url.trim().to_string());
                }
            }
        }
    }
    None
}

/// Parse repository name from a git remote URL.
///
/// Handles:
/// - `git@github.com:user/repo.git` -> `repo`
/// - `https://github.com/user/repo.git` -> `repo`
/// - `https://github.com/user/repo` -> `repo`
/// - `ssh://git@github.com/user/repo.git` -> `repo`
fn parse_repo_name(url: &str) -> Option<ProjectName> {
    let url = url.trim();

    // Get the last path component
    let name = if let Some(colon_pos) = url.rfind(':') {
        // SSH style: git@host:user/repo.git
        // But not ssh:// or https:// — those have :// before any path colon
        if url[..colon_pos].contains("://") {
            // It's a URL with scheme — extract last path segment
            url.rsplit('/').next()?
        } else {
            // SSH short form: git@host:user/repo.git
            let after_colon = &url[colon_pos + 1..];
            after_colon.rsplit('/').next()?
        }
    } else {
        url.rsplit('/').next()?
    };

    // Strip .git suffix
    let name = name.strip_suffix(".git").unwrap_or(name);

    name.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let name = parse_repo_name("git@github.com:user/my-repo.git").unwrap();
        assert_eq!(name.as_str(), "my-repo");
    }

    #[test]
    fn test_parse_https_url() {
        let name = parse_repo_name("https://github.com/user/my-repo.git").unwrap();
        assert_eq!(name.as_str(), "my-repo");
    }

    #[test]
    fn test_parse_https_no_git_suffix() {
        let name = parse_repo_name("https://github.com/user/my-repo").unwrap();
        assert_eq!(name.as_str(), "my-repo");
    }

    #[test]
    fn test_parse_ssh_scheme_url() {
        let name = parse_repo_name("ssh://git@github.com/user/my-repo.git").unwrap();
        assert_eq!(name.as_str(), "my-repo");
    }

    #[test]
    fn test_find_git_root() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        let root = find_git_root(&nested).unwrap();
        assert_eq!(root, tmp.path());
    }

    #[test]
    fn test_find_git_root_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        // No .git anywhere in the temp dir's own subtree,
        // but we can't guarantee the parent dirs don't have one,
        // so just test that our nested dir finds the right one.
        let nested = tmp.path().join("x");
        std::fs::create_dir(&nested).unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        let root = find_git_root(&nested).unwrap();
        assert_eq!(root, tmp.path());
    }

    #[test]
    fn test_parse_origin_url() {
        let config = r#"
[core]
    repositoryformatversion = 0
[remote "origin"]
    url = git@github.com:user/my-repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*
[branch "main"]
    remote = origin
"#;
        let url = parse_origin_url(config).unwrap();
        assert_eq!(url, "git@github.com:user/my-repo.git");
    }

    #[test]
    fn test_parse_origin_url_https() {
        let config = r#"
[remote "origin"]
    url = https://github.com/user/my-repo.git
"#;
        let url = parse_origin_url(config).unwrap();
        assert_eq!(url, "https://github.com/user/my-repo.git");
    }
}
