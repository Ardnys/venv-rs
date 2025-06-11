use std::{io, path::Path};

use venv::Venv;

use crate::app::App;

pub mod app;
pub mod event;
pub mod ui;
pub mod venv;

// fn main() -> color_eyre::Result<()> {
//     color_eyre::install()?;
//     let terminal = ratatui::init();
//     let result = App::new().run(terminal);
//     ratatui::restore();
//     result
// }

fn main() -> io::Result<()> {
    println!("hello");
    let uh = shellexpand::tilde("~/.virtualenvs/ptvision/");
    let p = Path::new(uh.as_ref());
    Venv::parse_from_dir(p)?;
    Ok(())
}
