use app::Output;
use clap::Parser;
use commands::Cli;

use crate::app::App;

pub mod app;
pub mod commands;
pub mod dir_size;
pub mod event;
pub mod settings;
pub mod ui;
pub mod venv;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    // TODO: unwrap or default to config ?
    // let venvs_dir = cli.venvs_dir.as_deref();
    if let Some(venvs_dir) = cli.venvs_dir.as_deref() {
        // println!("Virtual environment directory: {}", venvs_dir.display());
        let terminal = ratatui::init();
        let app = App::new(venvs_dir.to_owned());
        let result = app.run(terminal);
        ratatui::restore();

        let output = result?;
        match output {
            Output::VenvPath(path_buf) => println!("{}", path_buf.to_str().unwrap()),
            Output::Requirements(s) => println!("{}", s),
            Output::None => {}
        }

        Ok(())
    } else {
        Ok(())
    }
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
