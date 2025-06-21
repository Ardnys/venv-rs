use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    #[arg(short, long, value_name = "DIR")]
    pub venvs_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub kind: Option<Kind>,
}

#[derive(Subcommand, Debug)]
pub enum Kind {
    Venv {
        #[arg(short, long)]
        show_size: bool,
    },
}
