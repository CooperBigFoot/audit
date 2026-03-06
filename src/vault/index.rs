//! Persistent JSON index for fast entry lookups without scanning the vault.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::VaultError;
use crate::vault::entry::Entry;
use crate::vault::search::{list_entries, StoredEntry};

const INDEX_VERSION: u8 = 1;
const INDEX_FILENAME: &str = ".clog-index.json";

/// A single entry in the persistent index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: PathBuf,
    pub timestamp: String,
    pub project: String,
    pub entry_type: String,
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// The persistent index file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClogIndex {
    pub version: u8,
    pub entries: Vec<IndexEntry>,
}

impl ClogIndex {
    /// Create an empty index.
    pub fn empty() -> Self {
        Self {
            version: INDEX_VERSION,
            entries: Vec::new(),
        }
    }

    /// Load the index from the vault root. Returns an empty index if the file
    /// does not exist.
    pub fn load(vault_root: &Path) -> Result<Self, VaultError> {
        let path = vault_root.join(INDEX_FILENAME);
        if !path.exists() {
            debug!("no index file found, returning empty index");
            return Ok(Self::empty());
        }

        let content = std::fs::read_to_string(&path).map_err(|source| VaultError::WriteError {
            path: path.clone(),
            source,
        })?;

        let index: ClogIndex =
            serde_json::from_str(&content).map_err(|e| VaultError::IndexCorrupt {
                path: path.clone(),
                reason: e.to_string(),
            })?;

        if index.version != INDEX_VERSION {
            return Err(VaultError::IndexCorrupt {
                path,
                reason: format!(
                    "unsupported version {} (expected {})",
                    index.version, INDEX_VERSION
                ),
            });
        }

        debug!(count = index.entries.len(), "loaded index");
        Ok(index)
    }

    /// Save the index to the vault root.
    pub fn save(&self, vault_root: &Path) -> Result<(), VaultError> {
        let path = vault_root.join(INDEX_FILENAME);
        let content =
            serde_json::to_string_pretty(self).map_err(|e| VaultError::IndexCorrupt {
                path: path.clone(),
                reason: format!("failed to serialize index: {e}"),
            })?;

        std::fs::write(&path, content).map_err(|source| VaultError::WriteError {
            path,
            source,
        })?;

        debug!(count = self.entries.len(), "saved index");
        Ok(())
    }

    /// Append a new entry to the index from a just-written Entry and its file path.
    pub fn append(&mut self, entry: &Entry, file_path: &Path) {
        let fm = entry.frontmatter();
        self.entries.push(IndexEntry {
            path: file_path.to_path_buf(),
            timestamp: fm.timestamp,
            project: fm.project,
            entry_type: fm.entry_type,
            tags: fm.tags,
            title: fm.title,
            severity: fm.severity,
            session_id: fm.session_id,
        });
    }

    /// Rebuild the index by scanning all entries in the vault.
    pub fn rebuild(vault_root: &Path) -> Result<Self, VaultError> {
        info!("rebuilding index from vault");
        let stored = list_entries(vault_root)?;
        let entries = stored.into_iter().map(IndexEntry::from_stored).collect();

        let index = Self {
            version: INDEX_VERSION,
            entries,
        };

        index.save(vault_root)?;
        info!(count = index.entries.len(), "index rebuilt");
        Ok(index)
    }
}

impl IndexEntry {
    /// Convert a StoredEntry (read from disk) into an IndexEntry.
    fn from_stored(stored: StoredEntry) -> Self {
        Self {
            path: stored.path,
            timestamp: stored.frontmatter.timestamp,
            project: stored.frontmatter.project,
            entry_type: stored.frontmatter.entry_type,
            tags: stored.frontmatter.tags,
            title: stored.frontmatter.title,
            severity: stored.frontmatter.severity,
            session_id: stored.frontmatter.session_id,
        }
    }
}

/// Append a newly written entry to the on-disk index. Loads, appends, saves.
/// If the index is corrupt, rebuilds it first.
pub fn append_to_index(
    vault_root: &Path,
    entry: &Entry,
    file_path: &Path,
) -> Result<(), VaultError> {
    let mut index = match ClogIndex::load(vault_root) {
        Ok(idx) => idx,
        Err(VaultError::IndexCorrupt { path, reason }) => {
            warn!(path = %path.display(), reason = %reason, "index corrupt, rebuilding");
            ClogIndex::rebuild(vault_root)?
        }
        Err(e) => return Err(e),
    };

    index.append(entry, file_path);
    index.save(vault_root)?;
    Ok(())
}
