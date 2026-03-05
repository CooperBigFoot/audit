use anyhow::{Context, Result};

use crate::config::Config;
use crate::vault::Vault;
use crate::vault::search;

pub fn run(
    query: String,
    project: Option<String>,
    tags: Option<String>,
    limit: usize,
) -> Result<()> {
    let config = Config::load().context("failed to load config — run `audit init` first")?;
    let vault = Vault::new(&config.vault_path);

    let mut entries = search::list_entries(vault.root())?;

    if let Some(proj) = project {
        entries = search::filter_by_project(entries, &proj);
    }

    if let Some(tags_str) = tags {
        let tag_strs: Vec<String> = tags_str.split(',').map(|s| s.trim().to_lowercase()).collect();
        entries = search::filter_by_tags(entries, &tag_strs);
    }

    entries = search::search_entries(entries, &query);
    entries.truncate(limit);

    if entries.is_empty() {
        println!("No matching entries found.");
        return Ok(());
    }

    for entry in &entries {
        let fm = &entry.frontmatter;
        let date = fm.timestamp.split('T').next().unwrap_or(&fm.timestamp);
        let title = extract_title(&entry.content);
        println!("[{date}] ({}) [{}] {title}", fm.entry_type, fm.project);
    }

    Ok(())
}

fn extract_title(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return title.to_string();
        }
    }
    "(untitled)".to_string()
}
