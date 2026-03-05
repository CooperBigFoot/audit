use std::io::Read as _;

use anyhow::{Context, Result};
use tracing::info;

use crate::config::Config;
use crate::project::detect_project;
use crate::types::{EntryKind, ProjectName, Tag, parse_tags};
use crate::vault::Vault;
use crate::vault::entry::Entry;

pub fn run(
    title: String,
    body: Option<String>,
    tags: Option<String>,
    project: Option<String>,
) -> Result<()> {
    let config = Config::load().context("failed to load config — run `audit init` first")?;
    let vault = Vault::new(&config.vault_path);

    let project_name = resolve_project(project, &config)?;
    let tags = resolve_tags(tags, &config)?;
    let body = resolve_body(body)?;

    let mut builder = Entry::builder()
        .with_kind(EntryKind::Log)
        .with_project(project_name)
        .with_title(title)
        .with_tags(tags);

    if let Some(b) = body {
        builder = builder.with_body(b);
    }

    let entry = builder.build()?;
    let path = vault.write_entry(&entry)?;

    info!(path = %path.display(), "log entry written");
    println!("Entry written: {}", path.display());
    Ok(())
}

/// Resolve project from flag, git detection, or config default.
pub fn resolve_project(explicit: Option<String>, config: &Config) -> Result<ProjectName> {
    // 1. Explicit flag
    if let Some(name) = explicit {
        return name
            .parse::<ProjectName>()
            .map_err(|e| anyhow::anyhow!("{e}"));
    }

    // 2. Git auto-detection
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(name) = detect_project(&cwd) {
            return Ok(name);
        }
    }

    // 3. Config default
    if let Some(result) = config.default_project_name() {
        return result.map_err(|e| anyhow::anyhow!("{e}"));
    }

    anyhow::bail!(
        "cannot determine project — use --project flag, run from a git repo, or set defaults.project in config"
    )
}

/// Parse tags from flag + merge with config defaults.
pub fn resolve_tags(explicit: Option<String>, config: &Config) -> Result<Vec<Tag>> {
    let mut tags = Vec::new();

    // Config defaults
    for t in &config.default_tags {
        if let Ok(tag) = t.parse() {
            tags.push(tag);
        }
    }

    // Explicit tags
    if let Some(input) = explicit {
        let parsed = parse_tags(&input).context("invalid tags")?;
        tags.extend(parsed);
    }

    Ok(tags)
}

/// Resolve body from flag or stdin pipe.
pub fn resolve_body(explicit: Option<String>) -> Result<Option<String>> {
    if let Some(b) = explicit {
        return Ok(Some(b));
    }

    // Check if stdin is piped
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("failed to read from stdin")?;
        if !buf.trim().is_empty() {
            return Ok(Some(buf));
        }
    }

    Ok(None)
}
