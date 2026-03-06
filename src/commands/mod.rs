pub mod decision;
pub mod init;
pub mod log;
pub mod problem;
pub mod recent;
pub mod reindex;
pub mod search;
pub mod setup_project;
pub mod stats;
pub mod sync;

use anyhow::Result;

use crate::cli::Command;

pub fn dispatch(command: Command) -> Result<()> {
    match command {
        Command::Init { vault } => init::run(vault),
        Command::Log {
            title,
            body,
            tags,
            project,
        } => log::run(title, body, tags, project),
        Command::Decision {
            title,
            rationale,
            alternatives,
            body,
            tags,
            project,
        } => decision::run(title, rationale, alternatives, body, tags, project),
        Command::Problem {
            title,
            body,
            solution,
            severity,
            tags,
            project,
        } => problem::run(title, body, solution, severity, tags, project),
        Command::Recent {
            project,
            tags,
            any_tags,
            kind,
            since,
            until,
            severity,
            min_severity,
            session,
            limit,
            format,
        } => recent::run(project, tags, any_tags, kind, since, until, severity, min_severity, session, limit, format),
        Command::Search {
            query,
            project,
            tags,
            any_tags,
            kind,
            since,
            until,
            severity,
            min_severity,
            session,
            limit,
            format,
        } => search::run(query, project, tags, any_tags, kind, since, until, severity, min_severity, session, limit, format),
        Command::SetupProject { path, name } => setup_project::run(path, name),
        Command::Sync { continuous } => sync::run(continuous),
        Command::Reindex => reindex::run(),
        Command::Stats {
            project,
            since,
            until,
            format,
        } => stats::run(project, since, until, format),
        Command::Projects => recent::run_projects(),
    }
}
