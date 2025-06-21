use std::path::Path;

use anyhow::Result;
use venv::Venv;

use crate::app::App;

pub mod app;
pub mod event;
pub mod settings;
pub mod ui;
pub mod venv;

// fn main() -> color_eyre::Result<()> {
//     color_eyre::install()?;
//     let terminal = ratatui::init();
//     let result = App::new().run(terminal);
//     ratatui::restore();
//     result
// }

fn main() -> Result<()> {
    // let _pytorch_venv = shellexpand::tilde("~/.virtualenvs/ptvision/");
    // let _pytorch_venv_path = Path::new(pytorch_venv.as_ref());

    let expanded_venvs = shellexpand::tilde("~/.virtualenvs/");
    let venvs_path = Path::new(expanded_venvs.as_ref());

    let venvs = Venv::from_venvs_dir(venvs_path)?;
    for venv in venvs.iter() {
        println!("{}", venv.name);
        println!("{} packages", venv.packages.len());
        println!("------------");
        for (i, package) in venv.packages.iter().take(5).enumerate() {
            println!(
                "  {}. {} - version: {}",
                i + 1,
                package.name,
                package.version
            );
        }
        println!();
    }

    // let venv = Venv::from_path(p)?;
    // println!("Venv: {}, packages: {}", venv.name, venv.packages.len());
    /*
     * let venv_list = Venv::venvs_from_dir(directory_of_venvs);
     * let venv = Venv::new(single_venv_path);
     *
     * */
    Ok(())
}
