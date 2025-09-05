use std::{
    fs,
    path::{Path, PathBuf},
};

use app::Output;
use clap::Parser;
use color_eyre::{
    eyre::{Context, bail},
    owo_colors::OwoColorize,
};
use commands::Cli;
use dirs::cache_dir;
use settings::get_config;
use venv::{Venv, parser::parse_from_dir};
use venv_search::search_venvs;

use crate::{app::App, settings::Shell};

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

    // I usually use Git Bash on windows which works with bash style activation and paths
    // That's why I need this goofy setting
    let peak_git_bash_experience =
        cfg!(windows) && !matches!(shell, Shell::CMD | Shell::POWERSHELL);

    let kind = cli.kind;
    // TODO: tbh these paths could use Cow maybe?? I allocate a lot of memory for the same path
    let vec_venvs = match kind {
        commands::Kind::Venv { path } => {
            // try to get the venv from cache first
            if let Some(cached_path) = venv_path_to_cache_path(&path) {
                let venv = Venv::load_cache(&cached_path)?;
                vec![venv]
            } else {
                // parse it as is if cache fails
                [path]
                    .iter()
                    .filter_map(|p| parse_from_dir(p).ok())
                    .collect()
            }
        }
        commands::Kind::Search { path } => {
            let venv_paths = search_venvs(path);
            let mut venvs = Vec::with_capacity(venv_paths.len());

            for p in &venv_paths {
                if let Some(cached_path) = venv_path_to_cache_path(&p) {
                    let venv = Venv::load_cache(&cached_path)?;
                    venvs.push(venv);
                } else {
                    let venv = parse_from_dir(p).unwrap();
                    venvs.push(venv);
                }
            }
            venvs
        }
        commands::Kind::Venvs { path } => {
            let cache_path = cache_dir()
                .expect("Could not get cache dir")
                .join("venv_rs");
            let venvs =
                Venv::from_cache(&cache_path).wrap_err("Error while reading venvs from cache.")?;
            if !venvs.is_empty() {
                venvs
            } else if let Some(p) = path {
                Venv::from_venvs_dir(&p).wrap_err_with(|| {
                    format!("Error while reading venvs directory: {}", p.display())
                })?
            } else if let Some(path_str) = config.venvs_dir {
                let p = Path::new(&path_str);
                Venv::from_venvs_dir(p).wrap_err_with(|| {
                    format!("Error while reading venvs directory: {}", p.display())
                })?
            } else {
                bail!(
                    "Couldn't find a path to venvs directory in config. Provide it as an argument."
                )
            }
        }
        commands::Kind::ListShells => {
            println!(
                "{} {}",
                "Available shells:".bold().bright_blue(),
                Shell::variants().join(", ")
            );
            return Ok(());
        }
    };

    // TODO: config to run the TUI in stderr to allow pipes and stuff
    let terminal = ratatui::init();
    let app = App::new(vec_venvs);
    let result = app.run(terminal);
    ratatui::restore();

    let output = result?;
    match output {
        Output::VenvPath(path_buf) => {
            let mut activation_command = shell.activation(path_buf.to_string_lossy());

            // Apply platform-specific tweaks
            #[cfg(windows)]
            {
                if peak_git_bash_experience {
                    activation_command =
                        activation_command.replace("/", "\\").replace("\\", "\\\\");
                }
            }

            // Print the activation command with consistent styling
            let banner_bold = "  🐍 Activation command copied to clipboard:".bold();
            let banner = banner_bold.green();
            let activation_cmd_bold = activation_command.bold();
            let highlighted_cmd = activation_cmd_bold.yellow();

            #[cfg(target_os = "linux")]
            {
                if config.extra.use_xclip {
                    println!("\n{}\n\n {}", banner, highlighted_cmd);
                    clipboard::copy_to_clipboard(&activation_command)?;
                } else {
                    println!(
                        "{}\n{}\n{}",
                        "It looks like you don't have xclip enabled!".bold().red(),
                        "You should install and enable it for the best user experience.".green(),
                        activation_command
                    );
                }
            }

            #[cfg(windows)]
            {
                println!("\n{}\n\n {}", banner, highlighted_cmd);
                clipboard::copy_to_clipboard(&activation_command)?;
            }
        }

        Output::Requirements(s) => println!("{s}"),
        Output::None => {}
    }

    Ok(())
}

// TODO: fix this
fn venv_path_to_cache_path(p: &Path) -> Option<PathBuf> {
    let fname = p
        .file_name()
        .expect("Could not get the filename")
        .to_str()
        .unwrap();

    let cached_fname = format!("{fname}.bin");
    let cached_file = cache_dir()
        .expect("Could not get cache dir")
        .join("venv_rs")
        .join(cached_fname);

    fs::create_dir_all(&cached_file).expect("Could not create cached file");

    Some(cached_file)
}
