use std::path::PathBuf;

/// Errors from vault operations.
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    /// Returned when the configured vault directory does not exist.
    #[error("vault not found at {path}")]
    VaultNotFound { path: PathBuf },

    /// Returned when writing an entry file fails.
    #[error("failed to write entry to {path}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Returned when parsing an existing entry file fails.
    #[error("failed to parse entry at {path}: {reason}")]
    ParseError { path: PathBuf, reason: String },

    /// Returned when the index file is corrupt or has an unsupported version.
    #[error("index corrupt or unsupported at {path}: {reason}")]
    IndexCorrupt { path: PathBuf, reason: String },
}

/// Errors from config operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Returned when the config file does not exist.
    #[error("config not found at {path}")]
    NotFound { path: PathBuf },

    /// Returned when the config file cannot be parsed.
    #[error("invalid config at {path}")]
    Invalid {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    /// Returned when writing the config file fails.
    #[error("failed to write config to {path}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Errors from project detection.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// Returned when the current directory is not inside a git repo.
    #[error("not inside a git repository")]
    NotARepo,

    /// Returned when project name cannot be determined.
    #[error("cannot detect project name from {path}")]
    CannotDetect { path: PathBuf },
}

/// Errors from date filter parsing.
#[derive(Debug, thiserror::Error)]
pub enum DateFilterError {
    /// Returned when the date filter string cannot be parsed.
    #[error("invalid date filter: {input} (expected ISO date, datetime, or relative like 7d/2w/1m)")]
    InvalidFormat { input: String },
}
