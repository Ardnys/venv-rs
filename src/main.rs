use std::{
    io::Write,
    process::{Command, Stdio},
};

use app::Output;
use arboard::Clipboard;
use clap::Parser;
use color_eyre::{eyre::Context, owo_colors::OwoColorize};
use commands::Cli;
use settings::get_config;
use venv::{Venv, parser::parse_from_dir};
use venv_search::search_venvs;

use crate::app::App;

pub mod app;
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

    let config = get_config().expect("Failed to get config");

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
    // TODO: config to run the TUI in stderr to allow pipes and stuff
    let terminal = ratatui::init();
    let app = App::new(vec_venvs);
    let result = app.run(terminal);
    ratatui::restore();

    let output = result?;
    match output {
        Output::VenvPath(path_buf) => {
            let activation_command = config.shell.activation(path_buf.to_string_lossy());

            if cfg!(target_os = "linux") {
                if config.extra.use_xclip {
                    println!(
                        "{}\n\n {}",
                        "  🐍 Activation command copied to clipboard:"
                            .bold()
                            .green(),
                        activation_command.bold().yellow()
                    );
                    let mut xclip = Command::new("xclip")
                        .args(["-selection", "clipboard"])
                        .stdin(Stdio::piped())
                        .spawn()?;
                    if let Some(stdin) = xclip.stdin.as_mut() {
                        stdin.write_all(activation_command.as_bytes())?;
                    }
                    xclip.wait()?;
                } else {
                    println!(
                        "{}\n{}",
                        "It looks like you don't have xclip enabled!".bold().red(),
                        "You should install and enable it for the best user experience.".green(),
                    );

                    eprintln!("{activation_command}");
                    // that's how i copy directly to clipboard lol isn't this cursed
                    // cargo run -- venvs ~/.virtualenvs 2> >(xclip -selection clipboard)
                    // Clipboard::new()?.set().wait().text(activation_command)?;
                }
            } else {
                println!(
                    "{}\n\n {}",
                    "  🐍 Activation command copied to clipboard:"
                        .bold()
                        .green(),
                    activation_command.bold().yellow()
                );
                Clipboard::new()?.set_text(activation_command)?;
            }
        }
        Output::Requirements(s) => println!("{s}"),
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
