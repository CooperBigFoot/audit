use anyhow::{Context, Result};

use crate::config::Config;
use crate::types::OutputFormat;
use crate::vault::Vault;
use crate::vault::search::{self, StoredEntry};

pub fn run(
    project: Option<String>,
    tags: Option<String>,
    limit: usize,
    format: String,
) -> Result<()> {
    let config = Config::load().context("failed to load config — run `audit init` first")?;
    let vault = Vault::new(&config.vault_path);

    let format: OutputFormat = format
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{e}"))?;

    let mut entries = search::list_entries(vault.root())?;

    if let Some(proj) = project {
        entries = search::filter_by_project(entries, &proj);
    }

    if let Some(tags_str) = tags {
        let tag_strs: Vec<String> = tags_str.split(',').map(|s| s.trim().to_lowercase()).collect();
        entries = search::filter_by_tags(entries, &tag_strs);
    }

    entries.truncate(limit);

    if entries.is_empty() {
        println!("No entries found.");
        return Ok(());
    }

    match format {
        OutputFormat::Short => print_short(&entries),
        OutputFormat::Full => print_full(&entries),
        OutputFormat::Json => print_json(&entries)?,
    }

    Ok(())
}

pub fn run_projects() -> Result<()> {
    let config = Config::load().context("failed to load config — run `audit init` first")?;
    let vault = Vault::new(&config.vault_path);

    let entries = search::list_entries(vault.root())?;
    let projects = search::unique_projects(&entries);

    if projects.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    for p in &projects {
        println!("{p}");
    }

    Ok(())
}

fn print_short(entries: &[StoredEntry]) {
    for entry in entries {
        let fm = &entry.frontmatter;
        let ts = &fm.timestamp;
        let date = ts.split('T').next().unwrap_or(ts);
        let kind = &fm.entry_type;
        let project = &fm.project;
        let title = extract_title(&entry.content);

        println!("[{date}] ({kind}) [{project}] {title}");
    }
}

fn print_full(entries: &[StoredEntry]) {
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            println!("{}", "\u{2500}".repeat(60));
        }
        println!("{}", entry.content);
    }
}

fn print_json(entries: &[StoredEntry]) -> Result<()> {
    let json_entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "path": e.path.display().to_string(),
                "timestamp": e.frontmatter.timestamp,
                "project": e.frontmatter.project,
                "entry_type": e.frontmatter.entry_type,
                "tags": e.frontmatter.tags,
                "title": extract_title(&e.content),
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_entries)?);
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
