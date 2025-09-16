use std::{fs, path::PathBuf};

use config::Config;
use dirs::config_dir;
use serde::Deserialize;

use crate::shell::Shell;

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
