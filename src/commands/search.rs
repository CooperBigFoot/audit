use anyhow::{Context, Result};

use crate::commands::recent;
use crate::config::Config;
use crate::types::OutputFormat;
use crate::vault::Vault;
use crate::vault::filter::QueryFilter;
use crate::vault::search;

pub fn run(
    query: String,
    project: Option<String>,
    tags: Option<String>,
    any_tags: Option<String>,
    kind: Vec<String>,
    since: Option<String>,
    until: Option<String>,
    severity: Option<String>,
    min_severity: Option<String>,
    session: Option<String>,
    limit: usize,
    format: String,
) -> Result<()> {
    let config = Config::load().context("failed to load config — run `clog init` first")?;
    let vault = Vault::new(&config.vault_path);

    let format: OutputFormat = format
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{e}"))?;

    let filter = QueryFilter::from_cli_args(
        project, tags, any_tags, since, until, kind, severity, min_severity, session,
    )?;

    let entries = search::list_entries(vault.root())?;
    let entries = filter.apply(entries);
    let mut entries = search::search_entries(entries, &query);
    entries.truncate(limit);

    if entries.is_empty() {
        println!("No matching entries found.");
        return Ok(());
    }

    match format {
        OutputFormat::Short => recent::print_short(&entries),
        OutputFormat::Full => recent::print_full(&entries),
        OutputFormat::Json => recent::print_json(&entries)?,
    }

    Ok(())
}
