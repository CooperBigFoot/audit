use crate::types::EntryKind;
use crate::vault::entry::Entry;

/// Render an entry as Obsidian-compatible markdown.
pub fn render_entry(entry: &Entry) -> String {
    let mut out = String::new();

    // Frontmatter
    let fm = entry.frontmatter();
    let yaml = serde_yaml::to_string(&fm).expect("frontmatter serialization should not fail");
    out.push_str("---\n");
    out.push_str(&yaml);
    out.push_str("---\n\n");

    // Title
    match entry.kind {
        EntryKind::Decision => {
            out.push_str(&format!("# Decision: {}\n\n", entry.title));
        }
        EntryKind::Problem => {
            out.push_str(&format!("# Problem: {}\n\n", entry.title));
        }
        EntryKind::Log => {
            out.push_str(&format!("# {}\n\n", entry.title));
        }
    }

    // Project link
    out.push_str(&format!(
        "**Project:** [[projects/{}]]\n\n",
        entry.project.as_str()
    ));

    // Body / kind-specific sections
    match entry.kind {
        EntryKind::Log => render_log(entry, &mut out),
        EntryKind::Decision => render_decision(entry, &mut out),
        EntryKind::Problem => render_problem(entry, &mut out),
    }

    // Tags as hashtags at the end
    if !entry.tags.is_empty() {
        out.push('\n');
        let tag_line: Vec<String> = entry.tags.iter().map(|t| format!("#{}", t.as_str())).collect();
        out.push_str(&tag_line.join(" "));
        out.push('\n');
    }

    out
}

fn render_log(entry: &Entry, out: &mut String) {
    if let Some(body) = &entry.body {
        out.push_str("## Summary\n\n");
        out.push_str(body);
        out.push_str("\n\n");
    }
}

fn render_decision(entry: &Entry, out: &mut String) {
    if let Some(body) = &entry.body {
        out.push_str("## Context\n\n");
        out.push_str(body);
        out.push_str("\n\n");
    }

    out.push_str("## Decision\n\n");
    out.push_str(&format!("**Chose {}.**\n\n", entry.title));

    if let Some(rationale) = &entry.rationale {
        out.push_str("## Rationale\n\n");
        out.push_str(rationale);
        out.push_str("\n\n");
    }

    if !entry.alternatives.is_empty() {
        out.push_str("## Alternatives Considered\n\n");
        for alt in &entry.alternatives {
            out.push_str(&format!("- **{}**\n", alt));
        }
        out.push('\n');
    }
}

fn render_problem(entry: &Entry, out: &mut String) {
    if let Some(body) = &entry.body {
        out.push_str("## Symptom\n\n");
        out.push_str(body);
        out.push_str("\n\n");
    }

    if let Some(solution) = &entry.solution {
        out.push_str("## Solution\n\n");
        out.push_str(solution);
        out.push_str("\n\n");
    }
}
