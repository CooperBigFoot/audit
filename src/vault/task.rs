use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::VaultError;
use crate::types::{Priority, ProjectName, Tag, TaskId, TaskStatus};

/// YAML frontmatter for task markdown files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFrontmatter {
    pub id: u32,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub project: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub format_version: u8,
}

/// A kanban task.
#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub body: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub project: ProjectName,
    pub tags: Vec<Tag>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Task {
    /// Create a new task builder.
    pub fn builder() -> TaskBuilder {
        TaskBuilder::default()
    }

    /// Generate the filename for this task.
    pub fn filename(&self) -> String {
        format!("task-{:04}.md", self.id.as_u32())
    }

    /// Convert to YAML-serializable frontmatter.
    pub fn frontmatter(&self) -> TaskFrontmatter {
        TaskFrontmatter {
            id: self.id.as_u32(),
            title: self.title.clone(),
            status: self.status.to_string(),
            priority: self.priority.to_string(),
            project: self.project.as_str().to_string(),
            tags: self.tags.iter().map(|t| t.as_str().to_string()).collect(),
            created_at: self.created_at.to_rfc3339(),
            updated_at: self.updated_at.to_rfc3339(),
            format_version: 1,
        }
    }
}

/// Builder for constructing Task instances.
#[derive(Debug, Default)]
pub struct TaskBuilder {
    id: Option<TaskId>,
    title: Option<String>,
    body: Option<String>,
    status: Option<TaskStatus>,
    priority: Option<Priority>,
    project: Option<ProjectName>,
    tags: Vec<Tag>,
}

impl TaskBuilder {
    pub fn with_id(mut self, id: TaskId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_project(mut self, project: ProjectName) -> Self {
        self.project = Some(project);
        self
    }

    pub fn with_tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = tags;
        self
    }

    /// Build the Task. Returns Err if required fields are missing.
    pub fn build(self) -> anyhow::Result<Task> {
        let id = self
            .id
            .ok_or_else(|| anyhow::anyhow!("task id is required"))?;
        let title = self
            .title
            .ok_or_else(|| anyhow::anyhow!("task title is required"))?;
        let project = self
            .project
            .ok_or_else(|| anyhow::anyhow!("task project is required"))?;

        let now = Local::now();

        Ok(Task {
            id,
            title,
            body: self.body,
            status: self.status.unwrap_or(TaskStatus::Todo),
            priority: self.priority.unwrap_or(Priority::Medium),
            project,
            tags: self.tags,
            created_at: now,
            updated_at: now,
        })
    }
}

/// Render a task as Obsidian-compatible markdown.
pub fn render_task(task: &Task) -> String {
    let mut out = String::new();

    // Frontmatter
    let fm = task.frontmatter();
    let yaml = serde_yaml::to_string(&fm).expect("frontmatter serialization should not fail");
    out.push_str("---\n");
    out.push_str(&yaml);
    out.push_str("---\n\n");

    // Title
    out.push_str(&format!("# Task #{}: {}\n\n", task.id.as_u32(), task.title));

    // Project and status line
    out.push_str(&format!(
        "**Project:** [[projects/{}]]\n",
        task.project.as_str()
    ));
    out.push_str(&format!(
        "**Status:** {} | **Priority:** {}\n\n",
        task.status, task.priority
    ));

    // Description
    out.push_str("## Description\n\n");
    if let Some(body) = &task.body {
        out.push_str(body);
        out.push_str("\n\n");
    }

    // Tags as hashtags
    if !task.tags.is_empty() {
        let tag_line: Vec<String> = task.tags.iter().map(|t| format!("#{}", t.as_str())).collect();
        out.push_str(&tag_line.join(" "));
        out.push('\n');
    }

    out
}

/// A task read back from disk.
#[derive(Debug, Clone)]
pub struct StoredTask {
    pub path: PathBuf,
    pub frontmatter: TaskFrontmatter,
    pub content: String,
}

/// Parse a task file's YAML frontmatter.
pub fn parse_task_frontmatter(path: &Path) -> Result<StoredTask, VaultError> {
    let content = std::fs::read_to_string(path).map_err(|source| VaultError::WriteError {
        path: path.to_path_buf(),
        source,
    })?;

    let frontmatter = parse_frontmatter(&content, path)?;

    Ok(StoredTask {
        path: path.to_path_buf(),
        frontmatter,
        content,
    })
}

/// List all task files in the vault, sorted by id ascending.
pub fn list_task_files(vault_root: &Path) -> Result<Vec<StoredTask>, VaultError> {
    let tasks_dir = vault_root.join("tasks");
    if !tasks_dir.exists() {
        return Ok(Vec::new());
    }

    let read_dir = match std::fs::read_dir(&tasks_dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(Vec::new()),
    };

    let mut tasks = Vec::new();
    for entry in read_dir {
        let entry = entry.map_err(|source| VaultError::WriteError {
            path: tasks_dir.clone(),
            source,
        })?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "md") {
            match parse_task_frontmatter(&path) {
                Ok(stored) => tasks.push(stored),
                Err(e) => {
                    debug!(path = %path.display(), error = %e, "skipping unparseable task");
                }
            }
        }
    }

    // Sort by id ascending
    tasks.sort_by_key(|t| t.frontmatter.id);

    debug!(count = tasks.len(), "listed tasks");
    Ok(tasks)
}

fn parse_frontmatter(content: &str, path: &Path) -> Result<TaskFrontmatter, VaultError> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(VaultError::ParseError {
            path: path.to_path_buf(),
            reason: "no frontmatter delimiter found".to_string(),
        });
    }

    let after_first = &content[3..];
    let end = after_first
        .find("---")
        .ok_or_else(|| VaultError::ParseError {
            path: path.to_path_buf(),
            reason: "no closing frontmatter delimiter".to_string(),
        })?;

    let yaml_str = &after_first[..end];

    serde_yaml::from_str(yaml_str).map_err(|e| VaultError::ParseError {
        path: path.to_path_buf(),
        reason: format!("invalid YAML: {e}"),
    })
}
