use clap::{Parser, Subcommand};

/// Claude Code session journal for Obsidian.
#[derive(Debug, Parser)]
#[command(name = "clog", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize audit config and vault structure.
    Init {
        /// Path to the Obsidian vault root.
        #[arg(long)]
        vault: String,
    },

    /// Write a journal entry.
    Log {
        /// One-line description.
        #[arg(long)]
        title: String,

        /// Detailed body text. If omitted and stdin is a pipe, reads from stdin.
        #[arg(long)]
        body: Option<String>,

        /// Comma-separated tags.
        #[arg(long)]
        tags: Option<String>,

        /// Project name (auto-detected if omitted).
        #[arg(long)]
        project: Option<String>,
    },

    /// Record an architectural decision.
    Decision {
        /// What was decided.
        #[arg(long)]
        title: String,

        /// Why this was chosen.
        #[arg(long)]
        rationale: String,

        /// Rejected alternatives (can be repeated).
        #[arg(long = "alternative")]
        alternatives: Vec<String>,

        /// Context / background.
        #[arg(long)]
        body: Option<String>,

        /// Comma-separated tags.
        #[arg(long)]
        tags: Option<String>,

        /// Project name (auto-detected if omitted).
        #[arg(long)]
        project: Option<String>,
    },

    /// Record a problem and its solution.
    Problem {
        /// What went wrong.
        #[arg(long)]
        title: String,

        /// Symptom / description.
        #[arg(long)]
        body: Option<String>,

        /// How it was fixed.
        #[arg(long)]
        solution: Option<String>,

        /// Severity level.
        #[arg(long, default_value = "medium")]
        severity: String,

        /// Comma-separated tags.
        #[arg(long)]
        tags: Option<String>,

        /// Project name (auto-detected if omitted).
        #[arg(long)]
        project: Option<String>,
    },

    /// Show recent journal entries.
    Recent {
        /// Filter by project.
        #[arg(long)]
        project: Option<String>,

        /// Filter by tags (comma-separated).
        #[arg(long)]
        tags: Option<String>,

        /// Max entries to show.
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Output format: short, full, or json.
        #[arg(long, default_value = "short")]
        format: String,
    },

    /// Search journal entries.
    Search {
        /// Search query.
        query: String,

        /// Filter by project.
        #[arg(long)]
        project: Option<String>,

        /// Filter by tags (comma-separated).
        #[arg(long)]
        tags: Option<String>,

        /// Max results.
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Inject journaling instructions into a project's CLAUDE.md.
    SetupProject {
        /// Path to the project root.
        #[arg(long, default_value = ".")]
        path: String,

        /// Project name (auto-detected if omitted).
        #[arg(long)]
        name: Option<String>,
    },

    /// Sync vault via Obsidian CLI (ob).
    Sync {
        /// Run continuous sync.
        #[arg(long)]
        continuous: bool,
    },

    /// Rebuild project index pages from journal entries.
    Reindex,

    /// List known projects.
    Projects,
}
