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

impl FromStr for EntryKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "log" => Ok(Self::Log),
            "decision" => Ok(Self::Decision),
            "problem" => Ok(Self::Problem),
            other => Err(format!("invalid entry kind: {other}")),
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

impl Severity {
    pub fn rank(self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Critical => 3,
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

/// Monotonic task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskId(u32);

impl TaskId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TaskId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u32 = s
            .trim()
            .parse()
            .map_err(|e| format!("invalid task id: {e}"))?;
        Ok(Self(id))
    }
}

/// Kanban lifecycle state for tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    Done,
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backlog => f.write_str("backlog"),
            Self::Todo => f.write_str("todo"),
            Self::InProgress => f.write_str("in-progress"),
            Self::Done => f.write_str("done"),
            Self::Cancelled => f.write_str("cancelled"),
        }
    }
}

impl FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "backlog" => Ok(Self::Backlog),
            "todo" => Ok(Self::Todo),
            "in-progress" | "in_progress" | "inprogress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(format!(
                "invalid task status: {other} (expected backlog, todo, in-progress, done, or cancelled)"
            )),
        }
    }
}

impl TaskStatus {
    /// Returns true for open (non-terminal) statuses.
    pub fn is_open(self) -> bool {
        matches!(self, Self::Backlog | Self::Todo | Self::InProgress)
    }

    /// Numeric rank for sorting by lifecycle stage.
    pub fn rank(self) -> u8 {
        match self {
            Self::Backlog => 0,
            Self::Todo => 1,
            Self::InProgress => 2,
            Self::Done => 3,
            Self::Cancelled => 4,
        }
    }
}

/// Task urgency level (separate from problem severity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => f.write_str("low"),
            Self::Medium => f.write_str("medium"),
            Self::High => f.write_str("high"),
            Self::Critical => f.write_str("critical"),
        }
    }
}

impl FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            other => Err(format!(
                "invalid priority: {other} (expected low, medium, high, or critical)"
            )),
        }
    }
}

impl Priority {
    /// Numeric rank for sorting by urgency.
    pub fn rank(self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Critical => 3,
        }
    }
}

/// Output format for task list commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskOutputFormat {
    #[default]
    Board,
    Short,
    Full,
    Json,
}

impl fmt::Display for TaskOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Board => f.write_str("board"),
            Self::Short => f.write_str("short"),
            Self::Full => f.write_str("full"),
            Self::Json => f.write_str("json"),
        }
    }
}

impl FromStr for TaskOutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "board" => Ok(Self::Board),
            "short" => Ok(Self::Short),
            "full" => Ok(Self::Full),
            "json" => Ok(Self::Json),
            other => Err(format!(
                "invalid task output format: {other} (expected board, short, full, or json)"
            )),
        }
    }
}
