use std::collections::HashMap;

use serde::Serialize;

use crate::vault::search::StoredEntry;

/// A fully parsed and structured entry, suitable for JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedEntry {
    pub timestamp: String,
    pub project: String,
    pub kind: String,
    pub title: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,
    pub path: String,
}

/// Parse a StoredEntry into a fully structured ParsedEntry.
pub fn parse_stored_entry(entry: &StoredEntry) -> ParsedEntry {
    let fm = &entry.frontmatter;
    let title = extract_title(entry);
    let sections = extract_sections(&entry.content);

    let (body, rationale, alternatives, solution) = match fm.entry_type.as_str() {
        "decision" => (
            sections.get("Context").cloned(),
            sections.get("Rationale").cloned(),
            extract_list_items(sections.get("Alternatives Considered")),
            None,
        ),
        "problem" => (
            sections.get("Symptom").cloned(),
            None,
            Vec::new(),
            sections.get("Solution").cloned(),
        ),
        _ => (
            sections.get("Summary").cloned(),
            None,
            Vec::new(),
            None,
        ),
    };

    ParsedEntry {
        timestamp: fm.timestamp.clone(),
        project: fm.project.clone(),
        kind: fm.entry_type.clone(),
        title,
        tags: fm.tags.clone(),
        session_id: fm.session_id.clone(),
        severity: fm.severity.clone(),
        body,
        rationale,
        alternatives,
        solution,
        path: entry.path.display().to_string(),
    }
}

/// Extract title from frontmatter.title (preferred) or `# heading` (fallback).
pub fn extract_title(entry: &StoredEntry) -> String {
    if let Some(ref title) = entry.frontmatter.title {
        if !title.is_empty() {
            return title.clone();
        }
    }

    for line in entry.content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return title.to_string();
        }
    }

    "(untitled)".to_string()
}

/// Extract named `## Section` blocks from markdown body.
fn extract_sections(content: &str) -> HashMap<String, String> {
    let mut sections = HashMap::new();
    let mut current_section: Option<String> = None;
    let mut current_content = String::new();
    let mut in_frontmatter = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip frontmatter
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            continue;
        }

        // Skip H1 headings (title line)
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            continue;
        }

        // Detect H2 section
        if let Some(heading) = trimmed.strip_prefix("## ") {
            if let Some(ref name) = current_section {
                let content_trimmed = current_content.trim().to_string();
                if !content_trimmed.is_empty() {
                    sections.insert(name.clone(), content_trimmed);
                }
            }
            current_section = Some(heading.to_string());
            current_content = String::new();
        } else if current_section.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if let Some(ref name) = current_section {
        let content_trimmed = current_content.trim().to_string();
        if !content_trimmed.is_empty() {
            sections.insert(name.clone(), content_trimmed);
        }
    }

    sections
}

/// Extract list items from a section's content (lines starting with `- `).
fn extract_list_items(section: Option<&String>) -> Vec<String> {
    let Some(content) = section else {
        return Vec::new();
    };

    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let item = trimmed.strip_prefix("- ")?;
            let item = item.strip_prefix("**").unwrap_or(item);
            let item = item.strip_suffix("**").unwrap_or(item);
            Some(item.to_string())
        })
        .collect()
}
