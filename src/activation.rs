use std::path::PathBuf;

use color_eyre::{Result, owo_colors::OwoColorize};

use crate::{
    clipboard,
    settings::{Settings, Shell},
};

pub fn activation(shell: Shell, config: Settings, path_buf: PathBuf) -> Result<()> {
    // I usually use Git Bash on windows which works with bash style activation and paths
    // That's why I need this goofy setting
    #[cfg(windows)]
    let peak_git_bash_experience =
        cfg!(windows) && !matches!(shell, Shell::CMD | Shell::POWERSHELL);

    #[cfg(windows)]
    let mut activation_command = shell.activation(path_buf.to_string_lossy());

    #[cfg(target_os = "linux")]
    let activation_command = shell.activation(path_buf.to_string_lossy());

    // Apply platform-specific tweaks
    #[cfg(windows)]
    {
        if peak_git_bash_experience {
            activation_command = activation_command.replace("/", "\\").replace("\\", "\\\\");
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
            println!("\n{banner}\n\n {highlighted_cmd}");
            clipboard::copy_to_clipboard(&activation_command)
        } else {
            println!(
                "{}\n{}\n{}",
                "It looks like you don't have xclip enabled!".bold().red(),
                "You should install and enable it for the best user experience.".green(),
                activation_command
            );
            Ok(())
        }
    }

    #[cfg(windows)]
    {
        println!("\n{}\n\n {}", banner, highlighted_cmd);
        clipboard::copy_to_clipboard(&activation_command)?;
    }
}
