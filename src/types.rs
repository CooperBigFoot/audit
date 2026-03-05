use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A validated project name (lowercase, alphanumeric + hyphens).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectName(String);

impl ProjectName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ProjectName {
    type Err = ProjectNameError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        if s.is_empty() {
            return Err(ProjectNameError::Empty);
        }
        if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ProjectNameError::InvalidChars { name: s });
        }
        Ok(Self(s))
    }
}

impl fmt::Display for ProjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectNameError {
    #[error("project name cannot be empty")]
    Empty,
    #[error("project name contains invalid characters: {name} (only lowercase alphanumeric and hyphens allowed)")]
    InvalidChars { name: String },
}

/// A validated tag (lowercase, no spaces).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag(String);

impl Tag {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Tag {
    type Err = TagError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        if s.is_empty() {
            return Err(TagError::Empty);
        }
        if s.contains(' ') {
            return Err(TagError::ContainsSpaces { tag: s });
        }
        Ok(Self(s))
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("tag cannot be empty")]
    Empty,
    #[error("tag cannot contain spaces: {tag}")]
    ContainsSpaces { tag: String },
}

/// Parse a comma-separated string into a Vec of Tags.
pub fn parse_tags(input: &str) -> Result<Vec<Tag>, TagError> {
    input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse())
        .collect()
}

/// The kind of journal entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Log,
    Decision,
    Problem,
}

impl fmt::Display for EntryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Log => f.write_str("log"),
            Self::Decision => f.write_str("decision"),
            Self::Problem => f.write_str("problem"),
        }
    }
}

/// Severity level for problem entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => f.write_str("low"),
            Self::Medium => f.write_str("medium"),
            Self::High => f.write_str("high"),
            Self::Critical => f.write_str("critical"),
        }
    }
}

impl FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            other => Err(format!("invalid severity: {other} (expected low, medium, high, or critical)")),
        }
    }
}

/// Output format for read commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Short,
    Full,
    Json,
}

impl FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "short" => Ok(Self::Short),
            "full" => Ok(Self::Full),
            "json" => Ok(Self::Json),
            other => Err(format!("invalid format: {other} (expected short, full, or json)")),
        }
    }
}
