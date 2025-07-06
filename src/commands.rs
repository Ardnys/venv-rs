use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    /// Directory containing virtual environments
    #[arg(short, long, value_name = "DIR")]
    pub venvs_dir: Option<PathBuf>,

    /// Exit with output (for using with shell commands)
    #[arg(short)]
    pub output: bool,

    #[command(subcommand)]
    pub kind: Option<Kind>,
}

#[derive(Subcommand, Debug)]
pub enum Kind {
    /// Inspect a single virtual environment
    Venv {
        /// Path to virtual environment
        path: PathBuf,
        /// Show package sizes
        #[arg(short, long)]
        show_size: bool,
    },
}
