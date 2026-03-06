//! Query filtering for journal entries.

use std::str::FromStr;

use chrono::{DateTime, Duration, FixedOffset, Local, Offset};

use crate::error::DateFilterError;
use crate::types::{EntryKind, Severity};
use crate::vault::search::StoredEntry;

/// Filter criteria for querying journal entries.
#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    pub project: Option<String>,
    pub tags_all: Vec<String>,
    pub tags_any: Vec<String>,
    pub since: Option<DateTime<FixedOffset>>,
    pub until: Option<DateTime<FixedOffset>>,
    pub kinds: Vec<EntryKind>,
    pub severity: Option<Severity>,
    pub min_severity: Option<Severity>,
    pub session: Option<String>,
}

impl QueryFilter {
    /// Build from CLI args (raw strings). Parses dates, kinds, severity.
    pub fn from_cli_args(
        project: Option<String>,
        tags: Option<String>,
        any_tags: Option<String>,
        since: Option<String>,
        until: Option<String>,
        kind: Vec<String>,
        severity: Option<String>,
        min_severity: Option<String>,
        session: Option<String>,
    ) -> Result<Self, anyhow::Error> {
        let tags_all = tags
            .map(|t| t.split(',').map(|s| s.trim().to_lowercase()).collect())
            .unwrap_or_default();

        let tags_any = any_tags
            .map(|t| t.split(',').map(|s| s.trim().to_lowercase()).collect())
            .unwrap_or_default();

        let since = since.map(|s| parse_date_filter(&s)).transpose()?;
        let until = until.map(|s| parse_date_filter(&s)).transpose()?;

        let kinds: Vec<EntryKind> = kind
            .into_iter()
            .flat_map(|k| k.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
            .map(|k| EntryKind::from_str(&k).map_err(|e| anyhow::anyhow!("{e}")))
            .collect::<Result<Vec<_>, _>>()?;

        let severity = severity
            .map(|s| Severity::from_str(&s).map_err(|e| anyhow::anyhow!("{e}")))
            .transpose()?;

        let min_severity = min_severity
            .map(|s| Severity::from_str(&s).map_err(|e| anyhow::anyhow!("{e}")))
            .transpose()?;

        Ok(Self {
            project,
            tags_all,
            tags_any,
            since,
            until,
            kinds,
            severity,
            min_severity,
            session,
        })
    }

    /// Apply all filters to a vec of StoredEntry. Single pass.
    pub fn apply(&self, entries: Vec<StoredEntry>) -> Vec<StoredEntry> {
        entries.into_iter().filter(|e| self.matches(e)).collect()
    }

    /// Test one entry against all active filters.
    pub fn matches(&self, entry: &StoredEntry) -> bool {
        let fm = &entry.frontmatter;

        // Project filter
        if let Some(ref proj) = self.project {
            if fm.project != *proj {
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
            if let Ok(entry_dt) = DateTime::parse_from_rfc3339(&fm.timestamp) {
                if let Some(ref since) = self.since {
                    if entry_dt < *since {
                        return false;
                    }
                }
                if let Some(ref until) = self.until {
                    if entry_dt > *until {
                        return false;
                    }
                }
            }
        }

        // Kind filter
        if !self.kinds.is_empty() {
            if let Ok(entry_kind) = EntryKind::from_str(&fm.entry_type) {
                if !self.kinds.contains(&entry_kind) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Exact severity filter
        if let Some(ref sev) = self.severity {
            match &fm.severity {
                Some(s) => {
                    if let Ok(entry_sev) = Severity::from_str(s) {
                        if entry_sev != *sev {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Min severity filter
        if let Some(ref min_sev) = self.min_severity {
            match &fm.severity {
                Some(s) => {
                    if let Ok(entry_sev) = Severity::from_str(s) {
                        if entry_sev.rank() < min_sev.rank() {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Session filter
        if let Some(ref session) = self.session {
            match &fm.session_id {
                Some(sid) => {
                    if sid != session {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }
}

/// Parse "2026-02-01", "2026-02-01T14:00:00+00:00", "7d", "2w", "1m"
pub fn parse_date_filter(input: &str) -> Result<DateTime<FixedOffset>, DateFilterError> {
    let input = input.trim();

    // Try RFC 3339 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt);
    }

    // Try bare date YYYY-MM-DD
    if let Ok(naive) = chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        let local_offset = Local::now().offset().fix();
        let naive_dt = naive
            .and_hms_opt(0, 0, 0)
            .expect("midnight is always valid");
        let dt = naive_dt
            .and_local_timezone(local_offset)
            .single()
            .ok_or_else(|| DateFilterError::InvalidFormat {
                input: input.to_string(),
            })?;
        return Ok(dt);
    }

    // Try relative: Nd, Nw, Nm
    if input.len() >= 2 {
        let (num_str, suffix) = input.split_at(input.len() - 1);
        if let Ok(num) = num_str.parse::<i64>() {
            let duration = match suffix {
                "d" => Some(Duration::days(num)),
                "w" => Some(Duration::weeks(num)),
                "m" => Some(Duration::days(num * 30)),
                _ => None,
            };
            if let Some(dur) = duration {
                let now = Local::now();
                let past = now - dur;
                return Ok(past.fixed_offset());
            }
        }
    }

    Err(DateFilterError::InvalidFormat {
        input: input.to_string(),
    })
}
