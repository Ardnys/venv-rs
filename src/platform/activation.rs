use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use std::path::Path;

pub trait ShellActivator {
    fn activation_command(&self, path: &Path) -> Result<()>;
    // Print the activation command with consistent styling for all implementations
    fn pretty_print_activation_command(&self, cmd: &str) {
        let banner_bold = "  🐍 Activation command copied to clipboard:".bold();
        let banner = banner_bold.green();
        let activation_cmd_bold = cmd.bold();
        let highlighted_cmd = activation_cmd_bold.yellow();

        println!("\n{}\n\n {}", banner, highlighted_cmd);
    }
}

#[cfg(windows)]
mod act {
    use std::path::Path;

    use color_eyre::eyre::Result;

    use crate::{
        platform::{activation::ShellActivator, copy_to_clipboard},
        shell::Shell,
    };

    pub struct WindowsActivation {
        pub shell: Shell,
    }

    impl ShellActivator for WindowsActivation {
        fn activation_command(&self, path: &Path) -> Result<()> {
            let mut activation_command = self.shell.activation(path.to_string_lossy());

            if !matches!(self.shell, Shell::CMD | Shell::POWERSHELL) {
                activation_command = activation_command.replace("/", "\\").replace("\\", "\\\\");
            }

            self.pretty_print_activation_command(&activation_command);
            copy_to_clipboard(&activation_command)?;

            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
mod act {
    use std::path::Path;

    use color_eyre::eyre::Result;

    use crate::{
        platform::{activation::ShellActivator, copy_to_clipboard},
        shell::Shell,
    };

    pub struct LinuxActivation {
        pub shell: Shell,
        pub config: Settings,
    }

    // TODO: implement shell activation for linux
    impl ShellActivator for LinuxActivation {
        fn activation_command(&self, path: &Path) -> Result<()> {
            let activation_command = shell.activation(path_buf.to_string_lossy());

            if self.config.extra.use_xclip {
                self.pretty_print_activation_command(&activation_command);
                copy_to_clipboard(&activation_command)
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
    }
}

pub use act::*;
