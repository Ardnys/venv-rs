use std::{borrow::Cow, fs, path::PathBuf};

use config::Config;
use dirs::config_dir;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub shell: Shell,
    pub venvs_dir: Option<String>,
    pub extra: ExtraFeatures,
}

#[derive(Deserialize)]
pub struct ExtraFeatures {
    pub use_xclip: bool,
}

#[derive(Clone, Copy, Debug, Deserialize)]
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
    pub fn variants() -> &'static [&'static str] {
        &[
            "zsh",
            "bash",
            "fish",
            "csh",
            "tsch",
            "pwsh",
            "cmd",
            "powershell",
        ]
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

impl Settings {
    pub fn normalize_paths(mut self) -> Self {
        if let Some(venvs_dir) = &self.venvs_dir {
            let expanded = shellexpand::tilde(venvs_dir).into_owned();

            let canon = dunce::canonicalize(&expanded).unwrap_or(PathBuf::from(expanded));

            self.venvs_dir = Some(canon.to_string_lossy().into_owned());
        }
        self
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let config_dir = config_dir()
        .expect("Couldn't get config dir")
        .join("venv_rs");

    fs::create_dir_all(config_dir.as_path())
        .expect("Could not create config directories for some reason");

    let settings = Config::builder().set_default("venvs_dir", Option::<String>::None)?;

    let settings = if cfg!(target_os = "linux") {
        settings
            .set_default("shell", "zsh")?
            .set_default("extra.use_xclip", true)?
    } else {
        settings
            .set_default("shell", "cmd")?
            .set_default("extra.use_xclip", false)?
    };

    let settings = settings
        .add_source(config::File::from(config_dir.join("config.yaml")).required(false))
        .build()?;

    Ok(settings.try_deserialize::<Settings>()?.normalize_paths())
}
