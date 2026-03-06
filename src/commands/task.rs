use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use tracing::info;

use crate::cli::TaskAction;
use crate::commands::log::{resolve_body, resolve_project, resolve_tags};
use crate::config::Config;
use crate::types::{Priority, TaskId, TaskOutputFormat, TaskStatus, parse_tags};
use crate::vault::Vault;
use crate::vault::task::{
    StoredTask, Task, TaskFrontmatter, list_task_files, parse_task_frontmatter,
};
use crate::vault::task_filter::TaskQueryFilter;
use crate::vault::task_index::TaskIndex;

/// Dispatch a task subcommand.
pub fn dispatch_task(action: TaskAction) -> Result<()> {
    match action {
        TaskAction::Add {
            title,
            body,
            tags,
            project,
            priority,
            status,
        } => add(title, body, tags, project, priority, status),
        TaskAction::List {
            project,
            status,
            priority,
            tags,
            any_tags,
            all,
            format,
            limit,
        } => list(project, status, priority, tags, any_tags, all, format, limit),
        TaskAction::Show { id } => show(id),
        TaskAction::Update {
            id,
            title,
            body,
            priority,
            tags,
            status,
        } => update(id, title, body, priority, tags, status),
        TaskAction::Done { id } => done(id),
        TaskAction::Cancel { id } => cancel(id),
        TaskAction::Remove { id } => remove(id),
    }
}

fn add(
    title: String,
    body: Option<String>,
    tags: Option<String>,
    project: Option<String>,
    priority: String,
    status: String,
) -> Result<()> {
    let config = Config::load().context("failed to load config — run `clog init` first")?;
    let vault = Vault::new(&config.vault_path);

    let project_name = resolve_project(project, &config)?;
    let resolved_tags = resolve_tags(tags, &config)?;
    let resolved_body = resolve_body(body)?;

    let priority =
        Priority::from_str(&priority).map_err(|e| anyhow::anyhow!("{e}"))?;
    let status =
        TaskStatus::from_str(&status).map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut index = TaskIndex::load(vault.root()).context("failed to load task index")?;
    let id = index.next_task_id();
    let task_id = TaskId::new(id);

    let mut builder = Task::builder()
        .with_id(task_id)
        .with_title(&title)
        .with_project(project_name)
        .with_tags(resolved_tags)
        .with_priority(priority)
        .with_status(status);

    if let Some(b) = resolved_body {
        builder = builder.with_body(b);
    }

    let task = builder.build()?;
    let path = vault.write_task(&task).context("failed to write task")?;

    index.append(&task, &path);
    index.save(vault.root()).context("failed to save task index")?;

    info!(task_id = id, "task created");
    println!("Task #{id} created: {title}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn list(
    project: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    tags: Option<String>,
    any_tags: Option<String>,
    all: bool,
    format: String,
    limit: Option<usize>,
) -> Result<()> {
    let config = Config::load().context("failed to load config")?;
    let vault = Vault::new(&config.vault_path);

    let output_format =
        TaskOutputFormat::from_str(&format).map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut filter = TaskQueryFilter::from_cli_args(
        project, status, priority, tags, any_tags, None, None,
    )?;

    // Default to open tasks only unless --all or explicit status filter
    if !all && filter.statuses.is_empty() {
        filter.statuses = vec![TaskStatus::Backlog, TaskStatus::Todo, TaskStatus::InProgress];
    }

    let tasks = list_task_files(vault.root()).context("failed to list tasks")?;
    let mut tasks = filter.apply(tasks);

    if let Some(lim) = limit {
        tasks.truncate(lim);
    }

    match output_format {
        TaskOutputFormat::Board => print_board(&tasks),
        TaskOutputFormat::Short => print_short(&tasks),
        TaskOutputFormat::Full => print_full(&tasks),
        TaskOutputFormat::Json => print_json(&tasks)?,
    }

    Ok(())
}

fn show(id: u32) -> Result<()> {
    let config = Config::load().context("failed to load config")?;
    let vault = Vault::new(&config.vault_path);

    let path = vault.tasks_dir().join(format!("task-{id:04}.md"));
    let stored = parse_task_frontmatter(&path)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    print!("{}", stored.content);
    Ok(())
}

fn update(
    id: u32,
    title: Option<String>,
    body: Option<String>,
    priority: Option<String>,
    tags: Option<String>,
    status: Option<String>,
) -> Result<()> {
    let config = Config::load().context("failed to load config")?;
    let vault = Vault::new(&config.vault_path);

    let path = vault.tasks_dir().join(format!("task-{id:04}.md"));
    let stored = parse_task_frontmatter(&path)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut task = task_from_stored(&stored)?;

    if let Some(t) = &title {
        task.title = t.clone();
    }
    if let Some(b) = &body {
        task.body = Some(b.clone());
    }
    if let Some(p) = &priority {
        task.priority = Priority::from_str(p).map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    if let Some(t) = &tags {
        task.tags = parse_tags(t).context("invalid tags")?;
    }
    if let Some(s) = &status {
        task.status = TaskStatus::from_str(s).map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    task.updated_at = Local::now();

    let file_path = vault.update_task(&task).context("failed to update task")?;

    let mut index = TaskIndex::load(vault.root()).context("failed to load task index")?;
    index.update(&task, &file_path);
    index.save(vault.root()).context("failed to save task index")?;

    info!(task_id = id, "task updated");
    println!("Task #{id} updated: {}", task.title);
    Ok(())
}

fn done(id: u32) -> Result<()> {
    update(id, None, None, None, None, Some("done".to_string()))
}

fn cancel(id: u32) -> Result<()> {
    update(id, None, None, None, None, Some("cancelled".to_string()))
}

fn remove(id: u32) -> Result<()> {
    let config = Config::load().context("failed to load config")?;
    let vault = Vault::new(&config.vault_path);

    let task_id = TaskId::new(id);
    vault.remove_task(task_id).map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut index = TaskIndex::load(vault.root()).context("failed to load task index")?;
    index.remove(id);
    index.save(vault.root()).context("failed to save task index")?;

    info!(task_id = id, "task removed");
    println!("Task #{id} removed");
    Ok(())
}

/// Reconstruct a [`Task`] from a [`StoredTask`] by parsing frontmatter fields back to domain types.
fn task_from_stored(stored: &StoredTask) -> Result<Task> {
    let fm = &stored.frontmatter;

    let id = TaskId::new(fm.id);
    let status = TaskStatus::from_str(&fm.status).map_err(|e| anyhow::anyhow!("{e}"))?;
    let priority = Priority::from_str(&fm.priority).map_err(|e| anyhow::anyhow!("{e}"))?;
    let project = fm.project.parse().map_err(|e: crate::types::ProjectNameError| anyhow::anyhow!("{e}"))?;
    let tags: Vec<_> = fm
        .tags
        .iter()
        .filter_map(|t| t.parse().ok())
        .collect();

    let created_at = DateTime::parse_from_rfc3339(&fm.created_at)
        .context("invalid created_at timestamp")?
        .with_timezone(&Local);
    let updated_at = DateTime::parse_from_rfc3339(&fm.updated_at)
        .context("invalid updated_at timestamp")?
        .with_timezone(&Local);

    // Extract body from content: everything after "## Description\n\n" until the next section or tags
    let body = extract_body(&stored.content);

    Ok(Task {
        id,
        title: fm.title.clone(),
        body,
        status,
        priority,
        project,
        tags,
        created_at,
        updated_at,
    })
}

/// Extract the body text from the markdown content by finding the `## Description` section.
fn extract_body(content: &str) -> Option<String> {
    let marker = "## Description\n\n";
    let start = content.find(marker)?;
    let after = &content[start + marker.len()..];

    // Body ends at next heading, tag line (starts with #word), or end of file
    let end = after
        .find("\n## ")
        .or_else(|| {
            // Find a line that starts with hashtags (tag line)
            after.lines().enumerate().find_map(|(i, line)| {
                let trimmed = line.trim();
                if trimmed.starts_with('#') && !trimmed.starts_with("## ") {
                    Some(after.lines().take(i).map(|l| l.len() + 1).sum::<usize>())
                } else {
                    None
                }
            })
        })
        .unwrap_or(after.len());

    let body = after[..end].trim().to_string();
    if body.is_empty() {
        None
    } else {
        Some(body)
    }
}

// --- Output formatting ---

fn print_board(tasks: &[StoredTask]) {
    // All statuses in rank order
    let statuses = [
        TaskStatus::Backlog,
        TaskStatus::Todo,
        TaskStatus::InProgress,
        TaskStatus::Done,
        TaskStatus::Cancelled,
    ];

    let mut any_printed = false;

    for status in &statuses {
        let group: Vec<_> = tasks
            .iter()
            .filter(|t| TaskStatus::from_str(&t.frontmatter.status).ok().as_ref() == Some(status))
            .collect();

        if group.is_empty() {
            continue;
        }

        if any_printed {
            println!();
        }
        println!("=== {} ===", status.to_string().to_uppercase());
        for t in &group {
            let fm = &t.frontmatter;
            println!(
                "  #{:<4}  {:<30}  {:<8}  {}",
                fm.id, fm.title, fm.priority, fm.project
            );
        }
        any_printed = true;
    }

    if !any_printed {
        println!("No tasks found.");
    }
}

fn print_short(tasks: &[StoredTask]) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }
    for t in tasks {
        let fm = &t.frontmatter;
        println!(
            "#{} [{}] {} ({}, {})",
            fm.id, fm.status, fm.title, fm.priority, fm.project
        );
    }
}

fn print_full(tasks: &[StoredTask]) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }
    for (i, t) in tasks.iter().enumerate() {
        if i > 0 {
            println!("---");
        }
        print!("{}", t.content);
    }
}

fn print_json(tasks: &[StoredTask]) -> Result<()> {
    let items: Vec<&TaskFrontmatter> = tasks.iter().map(|t| &t.frontmatter).collect();
    let json = serde_json::to_string_pretty(&items).context("failed to serialize tasks")?;
    println!("{json}");
    Ok(())
}
