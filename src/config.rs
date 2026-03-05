use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::ConfigError;
use crate::types::ProjectName;

/// Application configuration persisted to `~/.config/audit/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the Obsidian vault root.
    pub vault_path: PathBuf,

    /// Whether to auto-sync after writes.
    #[serde(default)]
    pub auto_sync: bool,

    /// Default tags applied to every entry.
    #[serde(default)]
    pub default_tags: Vec<String>,

    /// Default project when auto-detection fails.
    #[serde(default)]
    pub default_project: Option<String>,
}

impl Config {
    /// Return the standard config file path: `~/.config/audit/config.toml`.
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("clog")
            .join("config.toml")
    }

    /// Load config from the default path.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::default_path();
        Self::load_from(&path)
    }

    /// Load config from a specific path.
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        debug!(path = %path.display(), "loading config");

        if !path.exists() {
            return Err(ConfigError::NotFound {
                path: path.to_path_buf(),
            });
        }

        let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::WriteError {
            path: path.to_path_buf(),
            source,
        })?;

        toml::from_str(&contents).map_err(|source| ConfigError::Invalid {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Save config to the default path, creating parent directories.
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::default_path();
        self.save_to(&path)
    }

    /// Save config to a specific path.
    pub fn save_to(&self, path: &Path) -> Result<(), ConfigError> {
        debug!(path = %path.display(), "saving config");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ConfigError::WriteError {
                path: path.to_path_buf(),
                source,
            })?;
        }

        let contents = toml::to_string_pretty(self).expect("config serialization should not fail");

        std::fs::write(path, contents).map_err(|source| ConfigError::WriteError {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(())
    }

    /// Get the default project as a parsed ProjectName, if set.
    pub fn default_project_name(&self) -> Option<Result<ProjectName, crate::types::ProjectNameError>> {
        self.default_project.as_ref().map(|s| s.parse())
    }
}
