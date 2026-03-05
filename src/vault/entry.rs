use chrono::{DateTime, Local};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::types::{EntryKind, ProjectName, Severity, Tag};

/// YAML frontmatter for an Obsidian entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub timestamp: String,
    pub project: String,
    pub entry_type: String,
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// A complete journal entry.
#[derive(Debug, Clone)]
pub struct Entry {
    pub kind: EntryKind,
    pub project: ProjectName,
    pub title: String,
    pub tags: Vec<Tag>,
    pub body: Option<String>,
    pub created_at: DateTime<Local>,
    // Decision-specific
    pub rationale: Option<String>,
    pub alternatives: Vec<String>,
    // Problem-specific
    pub solution: Option<String>,
    pub severity: Option<Severity>,
}

impl Entry {
    pub fn builder() -> EntryBuilder {
        EntryBuilder::default()
    }

    /// Generate the filename for this entry: `2026-03-05T14-30-00_a7f3.md`
    pub fn filename(&self) -> String {
        let ts = self.created_at.format("%Y-%m-%dT%H-%M-%S");
        let suffix = random_hex_suffix();
        format!("{ts}_{suffix}.md")
    }

    /// Generate the directory path components: `["journal", "2026", "03", "05"]`
    pub fn dir_components(&self) -> Vec<String> {
        vec![
            "journal".to_string(),
            self.created_at.format("%Y").to_string(),
            self.created_at.format("%m").to_string(),
            self.created_at.format("%d").to_string(),
        ]
    }

    /// Build frontmatter for YAML serialization.
    pub fn frontmatter(&self) -> Frontmatter {
        let alias = format!("{}: {}", self.project.as_str(), self.title);
        Frontmatter {
            timestamp: self.created_at.to_rfc3339(),
            project: self.project.as_str().to_string(),
            entry_type: self.kind.to_string(),
            tags: self.tags.iter().map(|t| t.as_str().to_string()).collect(),
            aliases: vec![alias],
            severity: self.severity.map(|s| s.to_string()),
        }
    }
}

fn random_hex_suffix() -> String {
    let mut rng = rand::thread_rng();
    let val: u16 = rng.gen_range(0..0xFFFF);
    format!("{val:04x}")
}

/// Builder for constructing Entry instances.
#[derive(Debug, Default)]
pub struct EntryBuilder {
    kind: Option<EntryKind>,
    project: Option<ProjectName>,
    title: Option<String>,
    tags: Vec<Tag>,
    body: Option<String>,
    rationale: Option<String>,
    alternatives: Vec<String>,
    solution: Option<String>,
    severity: Option<Severity>,
}

impl EntryBuilder {
    pub fn with_kind(mut self, kind: EntryKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn with_project(mut self, project: ProjectName) -> Self {
        self.project = Some(project);
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }

    pub fn with_alternative(mut self, alt: impl Into<String>) -> Self {
        self.alternatives.push(alt.into());
        self
    }

    pub fn with_alternatives(mut self, alts: Vec<String>) -> Self {
        self.alternatives = alts;
        self
    }

    pub fn with_solution(mut self, solution: impl Into<String>) -> Self {
        self.solution = Some(solution.into());
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Build the Entry. Returns Err if required fields are missing.
    pub fn build(self) -> anyhow::Result<Entry> {
        let kind = self
            .kind
            .ok_or_else(|| anyhow::anyhow!("entry kind is required"))?;
        let project = self
            .project
            .ok_or_else(|| anyhow::anyhow!("project is required"))?;
        let title = self
            .title
            .ok_or_else(|| anyhow::anyhow!("title is required"))?;

        Ok(Entry {
            kind,
            project,
            title,
            tags: self.tags,
            body: self.body,
            created_at: Local::now(),
            rationale: self.rationale,
            alternatives: self.alternatives,
            solution: self.solution,
            severity: self.severity,
        })
    }
}
