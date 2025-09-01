use std::{
    fs::{self},
    path::{Path, PathBuf},
    str::FromStr,
};

use color_eyre::Result;
use color_eyre::eyre;
use ratatui::widgets::{ListState, ScrollbarState};

use crate::venv::parser::parse_from_dir;

use super::parser::Metadata;

#[derive(Debug, Clone)]
pub struct Venv {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub packages: Vec<Package>, // Vec<Rc<Package>>
    pub num_dist_info_packages: i32,
    pub binaries: PathBuf,
    pub path: PathBuf,
    pub list_state: ListState,
    pub scroll_state: ScrollbarState,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct VenvList {
    pub venvs: Vec<Venv>,
    pub list_state: ListState,
    pub scroll_state: ScrollbarState,
}

impl Package {
    pub fn new(name: &str, version: &str, size: u64, metadata: Metadata) -> Self {
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
        path: PathBuf,
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
            path,
        }
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        parse_from_dir(path)
    }

    pub fn from_venvs_dir(path: &Path) -> Result<Vec<Self>> {
        if !path.is_dir() {
            return Err(eyre::eyre!("{} is not a directory", path.display()));
        }
        let venvs: Vec<Self> = fs::read_dir(path)?
            .filter_map(Result::ok)
            .map(|venv| venv.path())
            .filter_map(|v_pb| match parse_from_dir(&v_pb) {
                Ok(venv) => Some(venv),
                Err(err) => {
                    eprintln!("Failed to parse venv at {}: {:#}", v_pb.display(), err);
                    None
                }
            })
            .collect();

        Ok(venvs)
    }

    pub fn activation_path(&self) -> PathBuf {
        // PathBuf::from_str(&self.name).unwrap().join(&self.binaries)
        self.binaries.clone()
    }

    pub fn requirements(&self) -> PathBuf {
        // TODO: I'll try to recreate it properly
        // WARN: I know very hacky
        PathBuf::from_str(&self.name)
            .unwrap()
            .join(&self.binaries)
            .join(PathBuf::from_str("python").unwrap())
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
