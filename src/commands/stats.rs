use std::collections::BTreeMap;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::types::OutputFormat;
use crate::vault::Vault;
use crate::vault::filter::QueryFilter;
use crate::vault::search::{self, StoredEntry};
use crate::vault::task::{list_task_files, StoredTask};

pub fn run(
    project: Option<String>,
    since: Option<String>,
    until: Option<String>,
    format: String,
) -> Result<()> {
    let config = Config::load().context("failed to load config — run `clog init` first")?;
    let vault = Vault::new(&config.vault_path);

    let format: OutputFormat = format
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{e}"))?;

    let filter = QueryFilter::from_cli_args(
        project, None, None, since, until, vec![], None, None, None,
    )?;

    let entries = search::list_entries(vault.root())?;
    let entries = filter.apply(entries);

    if entries.is_empty() {
        println!("No entries found.");
        return Ok(());
    }

    let tasks = list_task_files(vault.root())
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let stats = compute_stats(&entries, &tasks);

    match format {
        OutputFormat::Short | OutputFormat::Full => print_stats_table(&stats),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
    }

    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct Stats {
    total: usize,
    by_kind: BTreeMap<String, usize>,
    by_project: BTreeMap<String, usize>,
    by_severity: BTreeMap<String, usize>,
    top_tags: Vec<(String, usize)>,
    task_total: usize,
    tasks_by_status: BTreeMap<String, usize>,
}

fn compute_stats(entries: &[StoredEntry], tasks: &[StoredTask]) -> Stats {
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_project: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_severity: BTreeMap<String, usize> = BTreeMap::new();
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();

    for entry in entries {
        let fm = &entry.frontmatter;

        *by_kind.entry(fm.entry_type.clone()).or_default() += 1;
        *by_project.entry(fm.project.clone()).or_default() += 1;

        if let Some(ref sev) = fm.severity {
            *by_severity.entry(sev.clone()).or_default() += 1;
        }

        for tag in &fm.tags {
            *tag_counts.entry(tag.clone()).or_default() += 1;
        }
    }

    let mut top_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    top_tags.sort_by(|a, b| b.1.cmp(&a.1));
    top_tags.truncate(10);

    let mut tasks_by_status: BTreeMap<String, usize> = BTreeMap::new();
    for task in tasks {
        *tasks_by_status
            .entry(task.frontmatter.status.clone())
            .or_default() += 1;
    }

    Stats {
        total: entries.len(),
        by_kind,
        by_project,
        by_severity,
        top_tags,
        task_total: tasks.len(),
        tasks_by_status,
    }
}

fn print_stats_table(stats: &Stats) {
    println!("Total entries: {}\n", stats.total);

    println!("By kind:");
    for (kind, count) in &stats.by_kind {
        println!("  {kind:<12} {count}");
    }

    println!("\nBy project:");
    for (project, count) in &stats.by_project {
        println!("  {project:<20} {count}");
    }

    if !stats.by_severity.is_empty() {
        println!("\nBy severity:");
        for (sev, count) in &stats.by_severity {
            println!("  {sev:<12} {count}");
        }
    }

    if !stats.top_tags.is_empty() {
        println!("\nTop tags:");
        for (tag, count) in &stats.top_tags {
            println!("  {tag:<20} {count}");
        }
    }

    if stats.task_total > 0 {
        println!("\nTasks: {}", stats.task_total);
        for (status, count) in &stats.tasks_by_status {
            println!("  {status:<12} {count}");
        }
    }
}
