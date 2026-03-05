use anyhow::{Context, Result};
use tracing::info;

use crate::commands::log::{resolve_body, resolve_project, resolve_tags};
use crate::config::Config;
use crate::types::EntryKind;
use crate::vault::Vault;
use crate::vault::entry::Entry;

pub fn run(
    title: String,
    rationale: String,
    alternatives: Vec<String>,
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
        .with_kind(EntryKind::Decision)
        .with_project(project_name)
        .with_title(title)
        .with_tags(tags)
        .with_rationale(rationale)
        .with_alternatives(alternatives);

    if let Some(b) = body {
        builder = builder.with_body(b);
    }

    let entry = builder.build()?;
    let path = vault.write_entry(&entry)?;

    info!(path = %path.display(), "decision entry written");
    println!("Decision recorded: {}", path.display());
    Ok(())
}
