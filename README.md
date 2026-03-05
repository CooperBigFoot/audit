# clog

A CLI that journals Claude Code sessions into an Obsidian vault. It captures what you built, decisions you made, and problems you hit — all as searchable, tagged Markdown notes.

## Install

```bash
cargo install --path .
```

## Setup

```bash
# Point clog at your Obsidian vault
clog init --vault ~/Documents/MyVault

# Inject journaling instructions into a project's CLAUDE.md
clog setup-project --path /path/to/project
```

Config lives at `~/.config/clog/config.toml`.

## Commands

| Command | Purpose |
|---|---|
| `clog log` | Write a journal entry |
| `clog decision` | Record an architectural decision |
| `clog problem` | Record a problem and its solution |
| `clog recent` | Show recent entries (filterable by project/tags) |
| `clog search <query>` | Full-text search across entries |
| `clog projects` | List known projects |
| `clog reindex` | Rebuild project index pages |
| `clog sync` | Sync vault via Obsidian CLI |

## Usage

```bash
# Log work
clog log --title "Add retry logic to API client" \
  --body "Exponential backoff with jitter, max 3 retries" \
  --tags "networking,reliability"

# Record a decision
clog decision --title "Use SQLite over Postgres" \
  --rationale "Single-user CLI, no need for a server" \
  --alternative "Postgres" --alternative "DuckDB"

# Record a problem
clog problem --title "Deadlock in worker pool" \
  --solution "Switch from Mutex to RwLock" \
  --severity high

# Review recent context
clog recent --limit 5 --project myapp
```

## Vault structure

Entries are stored as Markdown files with YAML frontmatter, organized by date:

```
vault/
  journal/
    2026/
      03/
        05/
          2026-03-05T14-30-00_a7f3.md
```

Each entry is tagged, timestamped, and linked to a project — making it searchable in Obsidian's graph view and through `clog search`.
