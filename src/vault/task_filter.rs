//! Query filtering for tasks.

use std::str::FromStr;

use chrono::{DateTime, FixedOffset};

use crate::types::{Priority, TaskStatus};
use crate::vault::filter::parse_date_filter;
use crate::vault::task::{StoredTask, TaskFrontmatter};

/// Filter criteria for querying tasks.
#[derive(Debug, Clone, Default)]
pub struct TaskQueryFilter {
    pub project: Option<String>,
    pub statuses: Vec<TaskStatus>,
    pub priorities: Vec<Priority>,
    pub tags_all: Vec<String>,
    pub tags_any: Vec<String>,
    pub since: Option<DateTime<FixedOffset>>,
    pub until: Option<DateTime<FixedOffset>>,
}

impl TaskQueryFilter {
    /// Build from CLI args (raw strings). Parses statuses, priorities, tags, dates.
    pub fn from_cli_args(
        project: Option<String>,
        status: Option<String>,
        priority: Option<String>,
        tags: Option<String>,
        any_tags: Option<String>,
        since: Option<String>,
        until: Option<String>,
    ) -> Result<Self, anyhow::Error> {
        let statuses: Vec<TaskStatus> = status
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .into_iter()
            .map(|s| TaskStatus::from_str(&s).map_err(|e| anyhow::anyhow!("{e}")))
            .collect::<Result<Vec<_>, _>>()?;

        let priorities: Vec<Priority> = priority
            .map(|p| {
                p.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .into_iter()
            .map(|p| Priority::from_str(&p).map_err(|e| anyhow::anyhow!("{e}")))
            .collect::<Result<Vec<_>, _>>()?;

        let tags_all = tags
            .map(|t| t.split(',').map(|s| s.trim().to_lowercase()).collect())
            .unwrap_or_default();

        let tags_any = any_tags
            .map(|t| t.split(',').map(|s| s.trim().to_lowercase()).collect())
            .unwrap_or_default();

        let since = since.map(|s| parse_date_filter(&s)).transpose()?;
        let until = until.map(|s| parse_date_filter(&s)).transpose()?;

        Ok(Self {
            project,
            statuses,
            priorities,
            tags_all,
            tags_any,
            since,
            until,
        })
    }

    /// Test one task frontmatter against all active filters.
    pub fn matches(&self, fm: &TaskFrontmatter) -> bool {
        // Project filter
        if let Some(ref proj) = self.project {
            if fm.project != *proj {
                return false;
            }
        }

        // Status filter
        if !self.statuses.is_empty() {
            if let Ok(task_status) = TaskStatus::from_str(&fm.status) {
                if !self.statuses.contains(&task_status) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Priority filter
        if !self.priorities.is_empty() {
            if let Ok(task_priority) = Priority::from_str(&fm.priority) {
                if !self.priorities.contains(&task_priority) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Tags ALL (AND logic)
        if !self.tags_all.is_empty() && !self.tags_all.iter().all(|t| fm.tags.contains(t)) {
            return false;
        }

        // Tags ANY (OR logic)
        if !self.tags_any.is_empty() && !self.tags_any.iter().any(|t| fm.tags.contains(t)) {
            return false;
        }

        // Date filters
        if self.since.is_some() || self.until.is_some() {
            if let Ok(task_dt) = DateTime::parse_from_rfc3339(&fm.created_at) {
                if let Some(ref since) = self.since {
                    if task_dt < *since {
                        return false;
                    }
                }
                if let Some(ref until) = self.until {
                    if task_dt > *until {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Apply all filters to a vec of StoredTask. Single pass.
    pub fn apply(&self, tasks: Vec<StoredTask>) -> Vec<StoredTask> {
        tasks.into_iter().filter(|t| self.matches(&t.frontmatter)).collect()
    }
}
