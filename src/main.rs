use activation::activation;
use app::Output;
use clap::Parser;
use commands::{Cli, handle_commands};
use settings::get_config;
use venv::model::VenvManager;

use crate::app::App;

pub mod activation;
pub mod app;
pub mod clipboard;
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

    let shell = cli
        .shell
        .and_then(|s| s.try_into().ok())
        .unwrap_or(config.shell);

    let mut vm = VenvManager::default();
    vm.load_cache().expect("Could not load the cache");

    let quit_early = handle_commands(&mut vm, &config)?;
    if quit_early {
        return Ok(());
    }

    // TODO: config to run the TUI in stderr to allow pipes and stuff
    let terminal = ratatui::init();
    let app = App::new(vm);
    let result = app.run(terminal);
    ratatui::restore();

    let output = result?;
    match output {
        Output::VenvPath(path_buf) => activation(shell, config, path_buf)?,
        Output::Requirements(s) => println!("{s}"),
        Output::None => {}
    }

    Ok(())
}
