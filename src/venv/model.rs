use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result, anyhow};
use color_eyre::owo_colors::OwoColorize;
use ratatui::widgets::ListState;

use crate::venv::parser::parse_from_dir;

// TODO: might add more details later
#[derive(Debug, Clone)]
pub struct Venv {
    pub name: String,
    // TODO: packages may be more complex later
    pub packages: Vec<Package>,
    pub state: ListState,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub size: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct VenvList {
    pub venvs: Vec<Venv>,
    pub state: ListState,
}

impl Package {
    pub fn new(name: &str, version: &str, metadata: HashMap<String, String>) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            size: 0.0,
            metadata,
        }
    }
}

impl Venv {
    pub fn from_path(path: &Path) -> Result<Self> {
        parse_from_dir(path)
    }

    pub fn new(name: &str, packages: Vec<Package>) -> Self {
        Self {
            name: name.to_string(),
            packages,
            state: ListState::default(),
        }
    }

    pub fn from_venvs_dir(path: &Path) -> Result<Vec<Self>> {
        if !path.is_dir() {
            return Err(anyhow!("{} is not a directory", path.display()));
        }
        let venvs: Vec<Self> = fs::read_dir(path)?
            .filter_map(Result::ok)
            .map(|venv| venv.path())
            .filter_map(|v_pb| parse_from_dir(&v_pb).ok())
            .collect();

        Ok(venvs)
    }
}

impl VenvList {
    pub fn new(venvs: Vec<Venv>) -> Self {
        Self {
            venvs,
            state: ListState::default(),
        }
    }
}

impl FromIterator<(&'static str, Vec<&'static str>)> for VenvList {
    fn from_iter<T: IntoIterator<Item = (&'static str, Vec<&'static str>)>>(iter: T) -> Self {
        let items = iter
            .into_iter()
            .map(|(name, packages)| {
                Venv::new(
                    name,
                    packages
                        .iter()
                        .map(|package| Package::new(package, "", HashMap::new()))
                        .collect(),
                )
            })
            .collect();
        Self {
            venvs: items,
            state: ListState::default(),
        }
    }
}
