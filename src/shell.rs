use std::borrow::Cow;

use serde::Deserialize;

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
