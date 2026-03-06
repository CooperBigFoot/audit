pub mod entry;
pub mod filter;
pub mod index;
pub mod markdown;
pub mod parser;
pub mod search;
pub mod task;
pub mod task_filter;
pub mod task_index;

use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};

use crate::error::VaultError;
use crate::types::TaskId;
use crate::vault::entry::Entry;
use crate::vault::index::append_to_index;
use crate::vault::markdown::render_entry;
use crate::vault::task::{render_task, Task};

/// Represents an Obsidian vault on disk.
#[derive(Debug, Clone)]
pub struct Vault {
    root: PathBuf,
}

impl Vault {
    /// Create a new Vault pointing to the given root directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Get the vault root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the tasks directory path.
    pub fn tasks_dir(&self) -> PathBuf {
        self.root.join("tasks")
    }

    /// Ensure the vault directory structure exists.
    pub fn ensure_dirs(&self) -> Result<(), VaultError> {
        let journal_dir = self.root.join("journal");
        let projects_dir = self.root.join("projects");
        let tasks_dir = self.tasks_dir();

        std::fs::create_dir_all(&journal_dir).map_err(|source| VaultError::WriteError {
            path: journal_dir,
            source,
        })?;
        std::fs::create_dir_all(&projects_dir).map_err(|source| VaultError::WriteError {
            path: projects_dir,
            source,
        })?;
        std::fs::create_dir_all(&tasks_dir).map_err(|source| VaultError::WriteError {
            path: tasks_dir,
            source,
        })?;

        Ok(())
    }

    /// Write an entry to the vault atomically.
    ///
    /// Uses tempfile + rename for atomic creation. Each entry gets a unique
    /// filename (timestamp + random suffix) so concurrent writes don't conflict.
    pub fn write_entry(&self, entry: &Entry) -> Result<PathBuf, VaultError> {
        let components = entry.dir_components();
        let mut dir = self.root.clone();
        for c in &components {
            dir.push(c);
        }

        std::fs::create_dir_all(&dir).map_err(|source| VaultError::WriteError {
            path: dir.clone(),
            source,
        })?;

        let filename = entry.filename();
        let target = dir.join(&filename);

        debug!(path = %target.display(), "writing entry");

        let content = render_entry(entry);

        // Write to a temp file in the same directory, then rename for atomicity
        let temp =
            tempfile::NamedTempFile::new_in(&dir).map_err(|source| VaultError::WriteError {
                path: target.clone(),
                source,
            })?;

        std::fs::write(temp.path(), &content).map_err(|source| VaultError::WriteError {
            path: target.clone(),
            source,
        })?;

        // persist + unique filenames is safe enough
        temp.persist(&target).map_err(|e| VaultError::WriteError {
            path: target.clone(),
            source: e.error,
        })?;

        info!(path = %target.display(), "entry written");

        // Update the persistent index
        if let Err(e) = append_to_index(&self.root, entry, &target) {
            warn!(error = %e, "failed to update index; it will be rebuilt on next read");
        }

        Ok(target)
    }

    /// Write a new task to the vault atomically.
    ///
    /// Uses tempfile + rename for atomic creation.
    pub fn write_task(&self, task: &Task) -> Result<PathBuf, VaultError> {
        let dir = self.tasks_dir();

        std::fs::create_dir_all(&dir).map_err(|source| VaultError::WriteError {
            path: dir.clone(),
            source,
        })?;

        let target = dir.join(task.filename());

        debug!(path = %target.display(), "writing task");

        let content = render_task(task);

        let temp =
            tempfile::NamedTempFile::new_in(&dir).map_err(|source| VaultError::WriteError {
                path: target.clone(),
                source,
            })?;

        std::fs::write(temp.path(), &content).map_err(|source| VaultError::WriteError {
            path: target.clone(),
            source,
        })?;

        temp.persist(&target).map_err(|e| VaultError::WriteError {
            path: target.clone(),
            source: e.error,
        })?;

        info!(path = %target.display(), "task written");

        Ok(target)
    }

    /// Update an existing task file by overwriting it.
    pub fn update_task(&self, task: &Task) -> Result<PathBuf, VaultError> {
        let target = self.tasks_dir().join(task.filename());

        debug!(path = %target.display(), "updating task");

        let content = render_task(task);

        std::fs::write(&target, &content).map_err(|source| VaultError::WriteError {
            path: target.clone(),
            source,
        })?;

        info!(path = %target.display(), "task updated");

        Ok(target)
    }

    /// Remove a task file from the vault.
    pub fn remove_task(&self, id: TaskId) -> Result<(), VaultError> {
        let filename = format!("task-{:04}.md", id.as_u32());
        let path = self.tasks_dir().join(filename);

        if !path.exists() {
            return Err(VaultError::TaskNotFound { id: id.as_u32() });
        }

        std::fs::remove_file(&path).map_err(|source| VaultError::WriteError {
            path,
            source,
        })?;

        info!(task_id = id.as_u32(), "task removed");

        Ok(())
    }
}
