use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{Ok, Result, eyre},
    owo_colors::OwoColorize,
};

use crate::{config::Settings, core::VenvManager, shell::Shell, venv::utils::search_venvs};
// use venv_rs_lib::{config::Settings, core::VenvManager, shell::Shell, venv::utils::search_venvs};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub kind: Kind,

    /// Shell for the activation command
    #[arg(short, long)]
    pub shell: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Kind {
    /// Inspect a single virtual environment
    Venv {
        /// Path to virtual environment
        path: PathBuf,
    },
    /// Search virtual environments recursively
    Search { path: PathBuf },
    /// Directory containing virtual environments
    Venvs { path: Option<PathBuf> },
    /// List available shells
    #[command(visible_alias = "ls")]
    ListShells,
}

pub fn handle_commands(vm: &mut VenvManager, config: &Settings) -> Result<bool> {
    let cli = Cli::parse();
    match cli.kind {
        Kind::Venv { path } => {
            let _ = vm.get(&path)?;
        }
        Kind::Search { path } => {
            let venv_paths = search_venvs(path);

            for p in &venv_paths {
                let _ = vm.get(p)?;
            }
        }
        Kind::Venvs { path } => {
            let p = path
                .or_else(|| config.venvs_dir.clone().map(PathBuf::from))
                .ok_or_else(|| {
                    eyre!("No path provided and venvs directory doesn't exit in config")
                })?;
            for res in fs::read_dir(&p)? {
                let entry = res?;
                let _ = vm.get(&entry.path())?;
            }
        }
        Kind::ListShells => {
            println!(
                "{} {}",
                "Available shells:".bold().bright_blue(),
                Shell::variants().join(", ")
            );
            return Ok(true);
        }
    };
    Ok(false)
}
