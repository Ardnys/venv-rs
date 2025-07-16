use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub kind: Kind,
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
    Venvs { path: PathBuf },
}
