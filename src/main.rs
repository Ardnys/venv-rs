use std::path::Path;

use anyhow::Result;
use clap::Parser;
use commands::Cli;
use venv::Venv;

use crate::app::App;

pub mod app;
pub mod commands;
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
        println!("Virtual environment directory: {}", venvs_dir.display());
        let terminal = ratatui::init();
        let result = App::new(venvs_dir).run(terminal);
        ratatui::restore();
        result
    } else {
        Ok(())
    }
}

/*
* actual usage:
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

// fn main() -> Result<()> {
//     // let _pytorch_venv = shellexpand::tilde("~/.virtualenvs/ptvision/");
//     // let _pytorch_venv_path = Path::new(pytorch_venv.as_ref());
//
//     let expanded_venvs = shellexpand::tilde("~/.virtualenvs/");
//     let venvs_path = Path::new(expanded_venvs.as_ref());
//
//     let venvs = Venv::from_venvs_dir(venvs_path)?;
//     for venv in venvs.iter() {
//         println!("{}", venv.name);
//         println!("{} packages", venv.packages.len());
//         println!("------------");
//         for (i, package) in venv.packages.iter().take(5).enumerate() {
//             println!(
//                 "  {}. {} - version: {}",
//                 i + 1,
//                 package.name,
//                 package.version
//             );
//         }
//         println!();
//     }
//     Ok(())
// }
