use std::path::Path;
use std::process::Command;

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

fn detect_from_remote(dir: &Path) -> Option<ProjectName> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_repo_name(&url)
}

fn detect_from_toplevel(dir: &Path) -> Option<ProjectName> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let toplevel = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let basename = Path::new(&toplevel).file_name()?.to_str()?;
    basename.parse().ok()
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
}
