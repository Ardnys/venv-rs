use std::{borrow::Cow, fs, path::PathBuf};

use config::Config;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub shell: Shell,
    pub extra: ExtraFeatures,
}

#[derive(Deserialize)]
pub struct ExtraFeatures {
    pub use_xclip: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    ZSH,
    BASH,
    FISH,
    CSH,
    TSCH,
    PWSH,
    CMD,
    POWERSHELL,
}

impl Shell {
    pub fn activation(&self, path_str: Cow<'_, str>) -> String {
        match self {
            Shell::ZSH | Shell::BASH => format!("source {path_str}/activate"),
            Shell::FISH => format!("source {path_str}/activate.fish"),
            Shell::CSH | Shell::TSCH => format!("source {path_str}/activate.csh"),
            Shell::PWSH => format!("{path_str}/Activate.ps1"),
            Shell::CMD => format!("{path_str}\\activate.bat"),
            Shell::POWERSHELL => format!("{path_str}\\Activate.ps1"),
        }
    }
}

impl TryFrom<String> for Shell {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "zsh" => Ok(Shell::ZSH),
            "bash" => Ok(Shell::BASH),
            "fish" => Ok(Shell::FISH),
            "csh" => Ok(Shell::CSH),
            "tsch" => Ok(Shell::TSCH),
            "cmd" => Ok(Shell::CMD),
            "powershell" => Ok(Shell::POWERSHELL),
            other => Err(format!(
                "{other} is not a valid shell. Use one of the following values: zsh, bash, fish, csh, tsch, cmd, powershell."
            )),
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let is_windows = cfg!(windows);
    let config_dir = if is_windows {
        todo!()
    } else {
        let home_path_str = std::env::var("HOME").expect("HOME environment variable must be set");
        PathBuf::from(home_path_str).join(".config").join("venv_rs")
    };

    fs::create_dir_all(config_dir.as_path())
        .expect("Could not create config directories for some reason");

    let settings = Config::builder()
        .set_default("shell", "zsh")?
        .set_default("extra.use_xclip", true)?
        .add_source(config::File::from(config_dir.join("config.yaml")).required(false))
        .build()?;

    settings.try_deserialize::<Settings>()
}
