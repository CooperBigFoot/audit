use anyhow::{Context, Result};
use tracing::info;

use crate::commands::log::{resolve_body, resolve_project, resolve_tags};
use crate::config::Config;
use crate::types::{EntryKind, Severity};
use crate::vault::Vault;
use crate::vault::entry::Entry;

pub fn run(
    title: String,
    body: Option<String>,
    solution: Option<String>,
    severity: String,
    tags: Option<String>,
    project: Option<String>,
) -> Result<()> {
    let config = Config::load().context("failed to load config — run `audit init` first")?;
    let vault = Vault::new(&config.vault_path);

    let project_name = resolve_project(project, &config)?;
    let tags = resolve_tags(tags, &config)?;
    let body = resolve_body(body)?;
    let severity: Severity = severity
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{e}"))?;

    let mut builder = Entry::builder()
        .with_kind(EntryKind::Problem)
        .with_project(project_name)
        .with_title(title)
        .with_tags(tags)
        .with_severity(severity);

    if let Some(b) = body {
        builder = builder.with_body(b);
    }
    if let Some(s) = solution {
        builder = builder.with_solution(s);
    }

    let entry = builder.build()?;
    let path = vault.write_entry(&entry)?;

    info!(path = %path.display(), "problem entry written");
    println!("Problem recorded: {}", path.display());
    Ok(())
}
