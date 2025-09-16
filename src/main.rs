use clap::Parser;
use venv_rs_lib::{
    commands::{Cli, handle_commands},
    config::get_config,
    core::VenvManager,
    platform::{ShellActivator, WindowsActivation},
    tui::{App, app::Output},
};

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

    vm.save_cache()?;

    // TODO: config to run the TUI in stderr to allow pipes and stuff
    let terminal = ratatui::init();
    let app = App::new(vm);
    let result = app.run(terminal);
    ratatui::restore();

    let output = result?;
    match output {
        Output::VenvPath(path_buf) => {
            #[cfg(windows)]
            let act = WindowsActivation { shell };

            #[cfg(target_os = "linux")]
            let act = LinuxActivation { shell, config };

            act.activation_command(&path_buf)?;
        }
        Output::Requirements(s) => println!("{s}"),
        Output::None => {}
    }

    Ok(())
}
