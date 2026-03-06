//! Persistent JSON index for fast task lookups.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::VaultError;
use crate::vault::task::{list_task_files, StoredTask, Task};

const TASK_INDEX_VERSION: u8 = 1;
const TASK_INDEX_FILENAME: &str = ".clog-task-index.json";

/// A single task summary stored in the persistent index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskIndexEntry {
    pub id: u32,
    pub path: PathBuf,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub project: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// The persistent task index file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskIndex {
    pub version: u8,
    pub next_id: u32,
    pub tasks: Vec<TaskIndexEntry>,
}

impl TaskIndex {
    /// Create an empty index.
    pub fn empty() -> Self {
        Self {
            version: TASK_INDEX_VERSION,
            next_id: 1,
            tasks: Vec::new(),
        }
    }

    /// Load the index from the vault root. Returns an empty index if the file
    /// does not exist.
    pub fn load(vault_root: &Path) -> Result<Self, VaultError> {
        let path = vault_root.join(TASK_INDEX_FILENAME);
        if !path.exists() {
            debug!("no task index file found, returning empty index");
            return Ok(Self::empty());
        }

        let content = std::fs::read_to_string(&path).map_err(|source| VaultError::WriteError {
            path: path.clone(),
            source,
        })?;

        let index: TaskIndex =
            serde_json::from_str(&content).map_err(|e| VaultError::IndexCorrupt {
                path: path.clone(),
                reason: e.to_string(),
            })?;

        if index.version != TASK_INDEX_VERSION {
            return Err(VaultError::IndexCorrupt {
                path,
                reason: format!(
                    "unsupported version {} (expected {})",
                    index.version, TASK_INDEX_VERSION
                ),
            });
        }

        debug!(count = index.tasks.len(), "loaded task index");
        Ok(index)
    }

    /// Save the index to the vault root.
    pub fn save(&self, vault_root: &Path) -> Result<(), VaultError> {
        let path = vault_root.join(TASK_INDEX_FILENAME);
        let content =
            serde_json::to_string_pretty(self).map_err(|e| VaultError::IndexCorrupt {
                path: path.clone(),
                reason: format!("failed to serialize task index: {e}"),
            })?;

        std::fs::write(&path, content).map_err(|source| VaultError::WriteError {
            path,
            source,
        })?;

        debug!(count = self.tasks.len(), "saved task index");
        Ok(())
    }

    /// Return the next available task id and increment the counter.
    pub fn next_task_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Append a new task entry to the index.
    pub fn append(&mut self, task: &Task, file_path: &Path) {
        let fm = task.frontmatter();
        self.tasks.push(TaskIndexEntry {
            id: fm.id,
            path: file_path.to_path_buf(),
            title: fm.title,
            status: fm.status,
            priority: fm.priority,
            project: fm.project,
            tags: fm.tags,
            created_at: fm.created_at,
            updated_at: fm.updated_at,
        });
    }

    /// Update an existing task entry in the index. If not found, appends instead.
    pub fn update(&mut self, task: &Task, file_path: &Path) {
        let fm = task.frontmatter();
        let new_entry = TaskIndexEntry {
            id: fm.id,
            path: file_path.to_path_buf(),
            title: fm.title,
            status: fm.status,
            priority: fm.priority,
            project: fm.project,
            tags: fm.tags,
            created_at: fm.created_at,
            updated_at: fm.updated_at,
        };

        if let Some(existing) = self.tasks.iter_mut().find(|e| e.id == new_entry.id) {
            *existing = new_entry;
        } else {
            warn!(id = new_entry.id, "task not found in index, appending");
            self.tasks.push(new_entry);
        }
    }

    /// Remove a task entry from the index by id.
    pub fn remove(&mut self, id: u32) {
        self.tasks.retain(|entry| entry.id != id);
    }

    /// Rebuild the index by scanning all task files in the vault.
    pub fn rebuild(vault_root: &Path) -> Result<Self, VaultError> {
        info!("rebuilding task index from vault");
        let stored = list_task_files(vault_root)?;

        let next_id = stored
            .iter()
            .map(|s| s.frontmatter.id)
            .max()
            .map(|max| max + 1)
            .unwrap_or(1);

        let tasks = stored.iter().map(TaskIndexEntry::from_stored).collect();

        let index = Self {
            version: TASK_INDEX_VERSION,
            next_id,
            tasks,
        };

        index.save(vault_root)?;
        info!(count = index.tasks.len(), next_id = index.next_id, "task index rebuilt");
        Ok(index)
    }

    /// Find a task entry by id.
    pub fn find_by_id(&self, id: u32) -> Option<&TaskIndexEntry> {
        self.tasks.iter().find(|entry| entry.id == id)
    }
}

impl TaskIndexEntry {
    /// Convert a [`StoredTask`] (read from disk) into a [`TaskIndexEntry`].
    fn from_stored(stored: &StoredTask) -> Self {
        Self {
            id: stored.frontmatter.id,
            path: stored.path.clone(),
            title: stored.frontmatter.title.clone(),
            status: stored.frontmatter.status.clone(),
            priority: stored.frontmatter.priority.clone(),
            project: stored.frontmatter.project.clone(),
            tags: stored.frontmatter.tags.clone(),
            created_at: stored.frontmatter.created_at.clone(),
            updated_at: stored.frontmatter.updated_at.clone(),
        }
    }
}
