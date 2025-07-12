use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Result, anyhow};
use ratatui::widgets::{ListState, ScrollbarState};

use crate::venv::parser::parse_from_dir;

// TODO: might add more details later
#[derive(Debug, Clone)]
pub struct Venv {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub packages: Vec<Package>,
    pub num_dist_info_packages: i32,
    pub binaries: PathBuf,
    pub list_state: ListState,
    pub scroll_state: ScrollbarState,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct VenvList {
    pub venvs: Vec<Venv>,
    pub list_state: ListState,
    pub scroll_state: ScrollbarState,
}

impl Package {
    pub fn new(name: &str, version: &str, size: u64, metadata: HashMap<String, String>) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            size,
            metadata,
        }
    }
}

impl Venv {
    pub fn new(
        name: &str,
        version: String,
        size: u64,
        packages: Vec<Package>,
        num_dist_info_packages: i32,
        binaries: PathBuf,
    ) -> Self {
        Self {
            name: name.to_string(),
            version,
            size,
            scroll_state: ScrollbarState::new(packages.len()),
            packages,
            num_dist_info_packages,
            list_state: ListState::default().with_selected(Some(0)),
            binaries,
        }
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        parse_from_dir(path)
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

    pub fn activation_path(&self) -> PathBuf {
        // TODO: can i check what kind of shell is active?
        // WARN: only supports linux rn
        PathBuf::from_str(&self.name)
            .unwrap()
            .join(&self.binaries)
            .join(PathBuf::from_str("activate").unwrap())
    }
}

impl VenvList {
    pub fn new(venvs: Vec<Venv>) -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            scroll_state: ScrollbarState::new(venvs.len()),
            venvs,
        }
    }
}

impl FromIterator<(&'static str, Vec<&'static str>)> for VenvList {
    fn from_iter<T: IntoIterator<Item = (&'static str, Vec<&'static str>)>>(iter: T) -> Self {
        let items: Vec<_> = iter
            .into_iter()
            .map(|(name, packages)| {
                Venv::new(
                    name,
                    "4.20".to_string(),
                    420,
                    packages
                        .iter()
                        .map(|package| Package::new(package, "", 0, HashMap::new()))
                        .collect(),
                    0,
                    PathBuf::from_str("").unwrap(),
                )
            })
            .collect();
        Self {
            scroll_state: ScrollbarState::new(items.len()),
            venvs: items,
            list_state: ListState::default(),
        }
    }
}
