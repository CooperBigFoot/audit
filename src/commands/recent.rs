use anyhow::{Context, Result};

use crate::config::Config;
use crate::types::OutputFormat;
use crate::vault::Vault;
use crate::vault::filter::QueryFilter;
use crate::vault::parser::{self, ParsedEntry};
use crate::vault::search::{self, StoredEntry};

pub fn run(
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
    let mut entries = filter.apply(entries);
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
    let config = Config::load().context("failed to load config — run `clog init` first")?;
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

pub fn print_short(entries: &[StoredEntry]) {
    for entry in entries {
        let fm = &entry.frontmatter;
        let ts = &fm.timestamp;
        let date = ts.split('T').next().unwrap_or(ts);
        let kind = &fm.entry_type;
        let project = &fm.project;
        let title = parser::extract_title(entry);

        println!("[{date}] ({kind}) [{project}] {title}");
    }
}

pub fn print_full(entries: &[StoredEntry]) {
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            println!("{}", "\u{2500}".repeat(60));
        }
        println!("{}", entry.content);
    }
}

pub fn print_json(entries: &[StoredEntry]) -> Result<()> {
    let parsed: Vec<ParsedEntry> = entries.iter().map(parser::parse_stored_entry).collect();
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    Ok(())
}
