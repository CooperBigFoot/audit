use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::info;

use crate::config::Config;
use crate::vault::Vault;

pub fn run(vault_path: String) -> Result<()> {
    let vault_path = PathBuf::from(vault_path);
    let vault_path = if vault_path.is_relative() {
        std::env::current_dir()?.join(vault_path)
    } else {
        vault_path
    };

    // Create vault directory structure
    let vault = Vault::new(&vault_path);
    vault
        .ensure_dirs()
        .context("failed to create vault directories")?;

    // Create config
    let config = Config {
        vault_path,
        auto_sync: false,
        default_tags: Vec::new(),
        default_project: None,
    };

    config.save().context("failed to save config")?;

    let config_path = Config::default_path();
    info!(vault = %config.vault_path.display(), config = %config_path.display(), "vault initialized");
    println!("Vault initialized at: {}", config.vault_path.display());
    println!("Config written to: {}", config_path.display());

    Ok(())
}
