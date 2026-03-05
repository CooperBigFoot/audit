use std::path::{Path, PathBuf};

use tracing::debug;

use crate::error::VaultError;
use crate::vault::entry::Frontmatter;

/// A parsed journal entry read back from disk.
#[derive(Debug, Clone)]
pub struct StoredEntry {
    pub path: PathBuf,
    pub frontmatter: Frontmatter,
    pub content: String,
}

/// List all journal entries in the vault, sorted by timestamp (newest first).
pub fn list_entries(vault_root: &Path) -> Result<Vec<StoredEntry>, VaultError> {
    let journal_dir = vault_root.join("journal");
    if !journal_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    walk_markdown_files(&journal_dir, &mut entries)?;

    // Sort by timestamp descending (newest first)
    entries.sort_by(|a, b| b.frontmatter.timestamp.cmp(&a.frontmatter.timestamp));

    debug!(count = entries.len(), "listed entries");
    Ok(entries)
}

/// Filter entries by project name.
pub fn filter_by_project(entries: Vec<StoredEntry>, project: &str) -> Vec<StoredEntry> {
    entries
        .into_iter()
        .filter(|e| e.frontmatter.project == project)
        .collect()
}

/// Filter entries by tags (entry must have ALL specified tags).
pub fn filter_by_tags(entries: Vec<StoredEntry>, tags: &[String]) -> Vec<StoredEntry> {
    entries
        .into_iter()
        .filter(|e| tags.iter().all(|t| e.frontmatter.tags.contains(t)))
        .collect()
}

/// Search entries by query string (case-insensitive match in content).
pub fn search_entries(entries: Vec<StoredEntry>, query: &str) -> Vec<StoredEntry> {
    let query_lower = query.to_lowercase();
    entries
        .into_iter()
        .filter(|e| e.content.to_lowercase().contains(&query_lower))
        .collect()
}

/// Collect unique project names from entries.
pub fn unique_projects(entries: &[StoredEntry]) -> Vec<String> {
    let mut projects: Vec<String> = entries
        .iter()
        .map(|e| e.frontmatter.project.clone())
        .collect();
    projects.sort();
    projects.dedup();
    projects
}

fn walk_markdown_files(dir: &Path, entries: &mut Vec<StoredEntry>) -> Result<(), VaultError> {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(()),
    };

    for entry in read_dir {
        let entry = entry.map_err(|source| VaultError::WriteError {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();

        if path.is_dir() {
            walk_markdown_files(&path, entries)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            match parse_stored_entry(&path) {
                Ok(stored) => entries.push(stored),
                Err(e) => {
                    debug!(path = %path.display(), error = %e, "skipping unparseable entry");
                }
            }
        }
    }

    Ok(())
}

fn parse_stored_entry(path: &Path) -> Result<StoredEntry, VaultError> {
    let content = std::fs::read_to_string(path).map_err(|source| VaultError::WriteError {
        path: path.to_path_buf(),
        source,
    })?;

    let frontmatter = parse_frontmatter(&content, path)?;

    Ok(StoredEntry {
        path: path.to_path_buf(),
        frontmatter,
        content,
    })
}

fn parse_frontmatter(content: &str, path: &Path) -> Result<Frontmatter, VaultError> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(VaultError::ParseError {
            path: path.to_path_buf(),
            reason: "no frontmatter delimiter found".to_string(),
        });
    }

    let after_first = &content[3..];
    let end = after_first
        .find("---")
        .ok_or_else(|| VaultError::ParseError {
            path: path.to_path_buf(),
            reason: "no closing frontmatter delimiter".to_string(),
        })?;

    let yaml_str = &after_first[..end];

    serde_yaml::from_str(yaml_str).map_err(|e| VaultError::ParseError {
        path: path.to_path_buf(),
        reason: format!("invalid YAML: {e}"),
    })
}
