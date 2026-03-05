pub mod decision;
pub mod init;
pub mod log;
pub mod problem;
pub mod recent;
pub mod reindex;
pub mod search;
pub mod setup_project;
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
            limit,
            format,
        } => recent::run(project, tags, limit, format),
        Command::Search {
            query,
            project,
            tags,
            limit,
        } => search::run(query, project, tags, limit),
        Command::SetupProject { path, name } => setup_project::run(path, name),
        Command::Sync { continuous } => sync::run(continuous),
        Command::Reindex => reindex::run(),
        Command::Projects => recent::run_projects(),
    }
}
