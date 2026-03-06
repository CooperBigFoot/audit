use std::collections::BTreeMap;

use anyhow::{Context, Result};
use tracing::info;

use crate::config::Config;
use crate::vault::Vault;
use crate::vault::index::ClogIndex;
use crate::vault::parser;
use crate::vault::search::{self, StoredEntry};
use crate::vault::task_index::TaskIndex;

pub fn run() -> Result<()> {
    let config = Config::load().context("failed to load config — run `audit init` first")?;
    let vault = Vault::new(&config.vault_path);
    let entries = search::list_entries(vault.root())?;

    if entries.is_empty() {
        println!("No entries found — nothing to reindex.");
        return Ok(());
    }

    // Group entries by project
    let mut by_project: BTreeMap<String, Vec<&StoredEntry>> = BTreeMap::new();
    for entry in &entries {
        by_project
            .entry(entry.frontmatter.project.clone())
            .or_default()
            .push(entry);
    }

    let projects_dir = config.vault_path.join("projects");
    std::fs::create_dir_all(&projects_dir)
        .context("failed to create projects directory")?;

    for (project, entries) in &by_project {
        let index_path = projects_dir.join(format!("{project}.md"));
        let content = build_project_index(project, entries);

        std::fs::write(&index_path, &content)
            .with_context(|| format!("failed to write index for project {project}"))?;

        info!(project = project, path = %index_path.display(), "rebuilt project index");
    }

    // Rebuild the persistent JSON index
    let index = ClogIndex::rebuild(vault.root())
        .context("failed to rebuild .clog-index.json")?;

    // Rebuild the task index
    let task_index = TaskIndex::rebuild(vault.root())
        .context("failed to rebuild .clog-task-index.json")?;

    println!(
        "Reindexed {} project(s), {} entries, {} tasks:",
        by_project.len(),
        index.entries.len(),
        task_index.tasks.len(),
    );
    for project in by_project.keys() {
        println!("  - {project}");
    }

    Ok(())
}

fn build_project_index(project: &str, entries: &[&StoredEntry]) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {project}\n\n"));
    out.push_str(&format!("*{} journal entries*\n\n", entries.len()));

    // Group by entry type
    let mut logs = Vec::new();
    let mut decisions = Vec::new();
    let mut problems = Vec::new();

    for entry in entries {
        match entry.frontmatter.entry_type.as_str() {
            "decision" => decisions.push(*entry),
            "problem" => problems.push(*entry),
            _ => logs.push(*entry),
        }
    }

    if !decisions.is_empty() {
        out.push_str("## Decisions\n\n");
        for entry in &decisions {
            write_entry_link(&mut out, entry);
        }
        out.push('\n');
    }

    if !problems.is_empty() {
        out.push_str("## Problems\n\n");
        for entry in &problems {
            write_entry_link(&mut out, entry);
        }
        out.push('\n');
    }

    if !logs.is_empty() {
        out.push_str("## Log\n\n");
        for entry in &logs {
            write_entry_link(&mut out, entry);
        }
        out.push('\n');
    }

    out
}

fn write_entry_link(out: &mut String, entry: &StoredEntry) {
    let date = entry.frontmatter.timestamp.split('T').next().unwrap_or("?");
    let title = parser::extract_title(entry);

    // Get the filename without extension for Obsidian wikilink
    let filename = entry.path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("?");

    out.push_str(&format!("- [{date}] [[journal/{filename}|{title}]]\n"));
}
