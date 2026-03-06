pub mod entry;
pub mod filter;
pub mod index;
pub mod markdown;
pub mod parser;
pub mod search;

use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};

use crate::error::VaultError;
use crate::vault::entry::Entry;
use crate::vault::index::append_to_index;
use crate::vault::markdown::render_entry;

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

    /// Ensure the vault directory structure exists.
    pub fn ensure_dirs(&self) -> Result<(), VaultError> {
        let journal_dir = self.root.join("journal");
        let projects_dir = self.root.join("projects");

        std::fs::create_dir_all(&journal_dir).map_err(|source| VaultError::WriteError {
            path: journal_dir,
            source,
        })?;
        std::fs::create_dir_all(&projects_dir).map_err(|source| VaultError::WriteError {
            path: projects_dir,
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
}
