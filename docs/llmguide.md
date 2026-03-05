# audit -- LLM Agent User Guide

`audit` is a CLI that writes structured journal entries (logs, decisions, problems) into an Obsidian vault. Multiple Claude Code instances across different projects can use it concurrently.

## Quick Reference

### Write Commands

| Command | Required Flags | Optional Flags | Purpose |
|---|---|---|---|
| `audit log` | `--title` | `--body`, `--tags`, `--project` | General journal entry |
| `audit decision` | `--title`, `--rationale` | `--alternative` (repeatable), `--body`, `--tags`, `--project` | Record an architectural decision |
| `audit problem` | `--title` | `--body`, `--solution`, `--severity` (default: `medium`), `--tags`, `--project` | Record a problem and fix |

### Read Commands

| Command | Arguments / Flags | Purpose |
|---|---|---|
| `audit recent` | `--project`, `--tags`, `--limit` (default: 10), `--format` (`short`/`full`/`json`) | Show recent entries |
| `audit search` | `<query>` (positional), `--project`, `--tags`, `--limit` (default: 20) | Full-text search |
| `audit projects` | (none) | List all known projects |

### Setup & Maintenance Commands

| Command | Arguments / Flags | Purpose |
|---|---|---|
| `audit init` | `--vault <path>` | Initialize config and vault structure |
| `audit setup-project` | `--path` (default: `.`), `--name` | Inject journaling instructions into CLAUDE.md |
| `audit sync` | `--continuous` | Run Obsidian CLI sync (`ob sync`) |
| `audit reindex` | (none) | Rebuild project index pages |

## Setup Flow

Run these once per machine / vault:

```bash
# 1. Initialize -- point to your Obsidian vault root
audit init --vault ~/Documents/ObsidianVault

# 2. In each project repo, inject journaling instructions into CLAUDE.md
cd /path/to/my-project
audit setup-project
```

`audit init` creates:
- Config at `~/.config/audit/config.toml`
- `journal/` and `projects/` directories inside the vault

`audit setup-project` appends a `<!-- BEGIN AUDIT JOURNALING -->` block to the project's `CLAUDE.md` (or creates one). It is idempotent -- running it again is a no-op.

## Core Workflow

### `audit log` -- general journal entry

Use after completing a unit of work (feature, refactor, investigation).

```bash
audit log --title "Implement retry logic for S3 uploads" \
  --body "Added exponential backoff with jitter. Max 5 retries. Timeout set to 30s per attempt." \
  --tags "s3,reliability"
```

### `audit decision` -- architectural decision record

Use when choosing between alternatives that affect the codebase long-term.

```bash
audit decision \
  --title "Use SQLite for local cache instead of Redis" \
  --rationale "No external daemon needed; single-file storage; sufficient for single-node workloads" \
  --alternative "Redis -- rejected due to deployment complexity" \
  --alternative "RocksDB -- rejected due to larger binary size" \
  --body "Evaluated caching options for the CLI's HTTP response cache" \
  --tags "architecture,caching"
```

The `--alternative` flag can be repeated for each rejected option.

### `audit problem` -- problem and solution record

Use when something breaks and you fix it (or when you hit a significant obstacle).

```bash
audit problem \
  --title "Cargo build fails on aarch64 with openssl-sys" \
  --body "Build error: 'openssl/sha.h' not found. Only happens on ARM macOS." \
  --solution "Switched to rustls-tls feature flag; removed openssl-sys dependency" \
  --severity high \
  --tags "build,macos"
```

Severity values: `low`, `medium` (default), `high`.

## Decision Tree -- When to Use Each Entry Type

```
Did you just complete a task or make progress?
  YES --> Was there a significant choice between alternatives?
    YES --> audit decision
    NO  --> Did something break or go wrong?
      YES --> audit problem
      NO  --> audit log
  NO  --> Do you need context before starting?
    YES --> audit recent / audit search
```

**Rules of thumb:**
- Default to `log`. It covers most situations.
- Use `decision` only when you actively chose between options and want the rationale preserved.
- Use `problem` when there was a failure, error, or unexpected obstacle -- even if the fix was trivial.

## Reading Back

### Recent entries

```bash
# Last 5 entries for the current project
audit recent --limit 5

# All recent entries in JSON (useful for programmatic consumption)
audit recent --format json --limit 20

# Filter by project and tags
audit recent --project hydra --tags "api,bugfix"
```

Output formats:
- `short` (default) -- one line per entry: `[date] (type) [project] title`
- `full` -- complete markdown content of each entry
- `json` -- structured JSON array with path, timestamp, project, entry_type, tags, title

### Search

```bash
audit search "retry logic" --project my-service
audit search "openssl" --tags "build" --limit 5
```

The query is a positional argument (not a flag). It performs full-text search across entry content.

### List projects

```bash
audit projects
```

Prints one project name per line. Useful to check what project names exist in the vault.

## Project Resolution

When `--project` is omitted, audit resolves the project name in this order:

1. **Git remote URL** -- parses the repo name from `git remote get-url origin` (e.g., `git@github.com:org/my-repo.git` becomes `my-repo`)
2. **Git toplevel directory** -- falls back to the basename of the repo root
3. **Config default** -- uses `default_project` from `~/.config/audit/config.toml`
4. **Error** -- if none of the above succeed, the command fails with a message to use `--project`

In practice, running audit from inside a git repo is sufficient. Use `--project` explicitly only when running from outside a repo or when you want to override the detected name.

## Stdin Body

When `--body` is omitted and stdin is a pipe, audit reads the body from stdin. This is useful for long-form content:

```bash
echo "Detailed description of what happened..." | audit log --title "Investigate memory leak"

# Pipe a file
cat investigation-notes.txt | audit problem --title "Memory leak in worker pool" --solution "Fixed unbounded channel"

# Pipe a command's output
git diff --stat HEAD~3 | audit log --title "Refactor authentication module"
```

If both `--body` and piped stdin are absent, the entry is created with no body.

## Vault Structure

All entries are written to the Obsidian vault specified during `audit init`.

```
vault-root/
  journal/
    2026/
      03/
        05/
          2026-03-05T14-30-00_a7f3.md
          2026-03-05T16-45-12_b1c2.md
  projects/
    my-repo.md        <-- project index page (built by reindex)
    other-project.md
```

- Entries are stored as `journal/YYYY/MM/DD/<timestamp>_<hex>.md`
- Each entry has YAML frontmatter (`timestamp`, `project`, `entry_type`, `tags`, `severity`, `aliases`)
- Entry body contains Obsidian wikilinks like `[[projects/my-repo]]`
- Tags appear as `#tagname` at the end of the file
- The random hex suffix in filenames prevents collisions from concurrent writes

### Project index pages

`audit reindex` generates `projects/<name>.md` files that link back to all journal entries for that project, grouped by type (Decisions, Problems, Log). These use Obsidian wikilinks for navigation.

## Sync and Reindex

```bash
# One-shot sync via Obsidian CLI
audit sync

# Continuous sync (keeps running)
audit sync --continuous

# Rebuild all project index pages from journal entries
audit reindex
```

`audit sync` shells out to the `ob` CLI (Obsidian CLI). It must be installed separately.

`audit reindex` scans all journal entries, groups them by project, and regenerates the project index pages under `projects/`. Run it after bulk operations or if index pages get out of date.

## Config File

Located at `~/.config/audit/config.toml`:

```toml
vault_path = "/Users/you/Documents/ObsidianVault"
auto_sync = false
default_tags = ["claude-code"]
default_project = "fallback-project"
```

- `default_tags` -- merged into every entry (in addition to any `--tags` flags)
- `default_project` -- used when git detection fails and `--project` is not provided
