use std::path::PathBuf;

use anyhow::{Context, Result};

const SENTINEL_BEGIN: &str = "<!-- BEGIN AUDIT JOURNALING -->";

const JOURNALING_BLOCK: &str = r#"<!-- BEGIN AUDIT JOURNALING -->
## Session Journaling (clog)

After completing meaningful work (feature, bug fix, investigation, refactoring), log it:

```bash
clog log --title "One-line description" \
  --body "Decisions made, problems encountered, key context" \
  --tags "tag1,tag2"
```

For architectural decisions:
```bash
clog decision --title "What was decided" --rationale "Why" --alternative "Option rejected"
```

For problems encountered:
```bash
clog problem --title "What broke" --solution "How it was fixed" --severity medium
```

Before starting work, check recent context:
```bash
clog recent --limit 5
```

For tracking tasks:
```bash
clog task add --title "What needs doing" --priority high --tags "tag1"
clog task list
clog task done 1
```
<!-- END AUDIT JOURNALING -->"#;

pub fn run(path: String, _name: Option<String>) -> Result<()> {
    let project_dir = PathBuf::from(path);
    let project_dir = if project_dir.is_relative() {
        std::env::current_dir()?.join(project_dir)
    } else {
        project_dir
    };

    let claude_md = project_dir.join("CLAUDE.md");

    // Check if CLAUDE.md exists and already has the sentinel
    if claude_md.exists() {
        let contents =
            std::fs::read_to_string(&claude_md).context("failed to read CLAUDE.md")?;

        if contents.contains(SENTINEL_BEGIN) {
            println!(
                "CLAUDE.md already contains audit journaling instructions (idempotent, no changes)"
            );
            return Ok(());
        }

        // Append to existing file
        let mut new_contents = contents;
        if !new_contents.ends_with('\n') {
            new_contents.push('\n');
        }
        new_contents.push('\n');
        new_contents.push_str(JOURNALING_BLOCK);
        new_contents.push('\n');

        std::fs::write(&claude_md, new_contents).context("failed to write CLAUDE.md")?;
    } else {
        // Create new CLAUDE.md
        let contents = format!("# Claude Agent Guidelines\n\n{JOURNALING_BLOCK}\n");
        std::fs::write(&claude_md, contents).context("failed to create CLAUDE.md")?;
    }

    println!(
        "Journaling instructions added to: {}",
        claude_md.display()
    );
    Ok(())
}
