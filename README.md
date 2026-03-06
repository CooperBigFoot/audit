# clog

A CLI that journals Claude Code sessions into an Obsidian vault. It captures what you built, decisions you made, problems you hit, and tasks you need to do — all as searchable, tagged Markdown notes.

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
| `clog task` (alias `t`) | Manage tasks with kanban-style tracking |
| `clog projects` | List known projects |
| `clog stats` | Show statistics about entries and tasks |
| `clog reindex` | Rebuild project and task index |
| `clog sync` | Sync vault via Obsidian CLI |

## Usage

### Journaling

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

### Task tracking

```bash
# Add a task
clog task add --title "Implement caching" --priority high --tags "performance"

# List open tasks (board view, default)
clog task list

# List all tasks including done/cancelled
clog task list --all

# Other formats
clog task list --format short
clog task list --format json

# Show a task
clog task show 1

# Update a task
clog task update 1 --status in-progress
clog task update 1 --priority critical --tags "performance,urgent"

# Mark done or cancel
clog task done 1
clog task cancel 2

# Remove a task permanently
clog task rm 3
```

#### Task statuses

`backlog` → `todo` → `in-progress` → `done` / `cancelled`

#### Task priorities

`low` | `medium` (default) | `high` | `critical`

## Vault structure

Entries and tasks are stored as Markdown files with YAML frontmatter:

```
vault/
  journal/
    2026/
      03/
        05/
          2026-03-05T14-30-00_a7f3.md
  tasks/
    task-0001.md
    task-0002.md
  projects/
    myapp.md
  .clog-index.json
  .clog-task-index.json
```

Each entry and task is tagged, timestamped, and linked to a project — making it searchable in Obsidian's graph view and through `clog search`.
