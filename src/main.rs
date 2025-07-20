use app::Output;
use clap::Parser;
use color_eyre::eyre::Context;
use comfy_table::create_comfy_table;
use commands::Cli;
use venv::{Venv, parser::parse_from_dir};
use venv_search::search_venvs;

use crate::app::App;

pub mod app;
pub mod comfy_table;
pub mod commands;
pub mod dir_size;
pub mod event;
pub mod settings;
pub mod ui;
pub mod venv;
pub mod venv_search;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    // TODO: unwrap or default to config ?
    // println!("Virtual environment directory: {}", venvs_dir.display());

    let kind = cli.kind;
    // TODO: tbh these paths could use Cow maybe?? I allocate a lot of memory for the same path
    let vec_venvs = match kind {
        commands::Kind::Venv { path } => [path]
            .iter()
            .filter_map(|p| parse_from_dir(p).ok())
            .collect(),
        commands::Kind::Search { path } => search_venvs(path)
            .iter()
            .filter_map(|p| parse_from_dir(p).ok())
            .collect(),
        commands::Kind::Venvs { path } => Venv::from_venvs_dir(&path).wrap_err_with(|| {
            format!(
                "Error while reading venvs directory: {}",
                path.to_string_lossy()
            )
        })?,
    };
    let terminal = ratatui::init();
    let app = App::new(vec_venvs);
    let result = app.run(terminal);
    ratatui::restore();

    let output = result?;
    match output {
        Output::VenvPath(path_buf) => {
            let path_str = path_buf.to_string_lossy();

            let table = create_comfy_table(path_str);

            println!("{table}");
            // println!(
            //     "{}\n\n {}  {} {}\n",
            //     "  🐍 To activate your virtualenv:".bold().green(),
            //     "  Linux".yellow().bold(),
            //     "source".bold(),
            //     path_str.bold(),
            // );
        }
        Output::Requirements(s) => println!("{}", s),
        Output::None => {}
    }

    Ok(())
}

/*
* planned usage:
* opens the virtualenvs directory from config (default: ~/.virtualenvs/)
* $ vem
*
* or you could provide it as a flag
* $ vem --venvs_dir <DIR>
*
* or you can inspect a single virtual environment
* vem venv <PATH>
*
* then we have flags like show size etc. but you should just use the config file
*/
