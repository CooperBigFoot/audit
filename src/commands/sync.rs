use std::process::Command;

use anyhow::{Context, Result};

pub fn run(continuous: bool) -> Result<()> {
    let mut cmd = Command::new("ob");
    cmd.arg("sync");

    if continuous {
        cmd.arg("--continuous");
    }

    println!("Running: ob sync{}", if continuous { " --continuous" } else { "" });

    let status = cmd.status().context(
        "failed to run `ob` — is Obsidian CLI installed? See https://obsidian.md/cli"
    )?;

    if !status.success() {
        anyhow::bail!("ob sync exited with status {status}");
    }

    Ok(())
}
