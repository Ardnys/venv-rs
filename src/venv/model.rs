use std::{
    fs::{self},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::SystemTime,
};

use bincode::{Decode, Encode, config};
use chrono::{DateTime, Local};
use color_eyre::Result;
use color_eyre::eyre;
use dirs::cache_dir;
use ratatui::widgets::{ListState, ScrollbarState};

use crate::venv::metadata::Metadata;

use super::parser::VenvParser;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Venv {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub packages: Vec<Package>,
    pub num_dist_info_packages: i32,
    pub binaries: PathBuf,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct VenvUi {
    pub venv: Arc<Venv>,
    pub list_state: ListState,
    pub scroll_state: ScrollbarState,
    pub last_modified: DateTime<Local>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub metadata: Metadata,
    pub last_modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct VenvListUi {
    pub venvs: Vec<VenvUi>,
    pub list_state: ListState,
    pub scroll_state: ScrollbarState,
}

impl Package {
    pub fn new(
        name: &str,
        version: &str,
        size: u64,
        metadata: Metadata,
        last_modified: SystemTime,
    ) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            size,
            metadata,
            last_modified,
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
            packages,
            num_dist_info_packages,
            binaries,
            path,
        }
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        VenvParser::parse_from_dir(path.to_path_buf())
    }

    pub fn from_venvs_dir(path: &Path) -> Result<Vec<Self>> {
        if !path.is_dir() {
            return Err(eyre::eyre!("{} is not a directory", path.display()));
        }
        let venvs: Vec<Self> = fs::read_dir(path)?
            .filter_map(Result::ok)
            .map(|venv| venv.path())
            .filter_map(
                |v_pb| match VenvParser::parse_from_dir(v_pb.to_path_buf()) {
                    Ok(venv) => Some(venv),
                    Err(err) => {
                        eprintln!("Failed to parse venv at {}: {:#}", v_pb.display(), err);
                        None
                    }
                },
            )
            .collect();

        Ok(venvs)
    }

    pub fn from_cache(path: &Path) -> Result<Vec<Self>> {
        fs::read_dir(path)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .map(|path| Venv::load_cache(&path))
            .collect()
    }

    fn cache_path(&self) -> PathBuf {
        let filename = format!("{}.bin", self.name);
        cache_dir()
            .expect("Could not get cache dir")
            .join("venv_rs")
            .join(filename)
    }

    pub fn save_cache_to(&self, cache_path: &Path) -> Result<()> {
        let config = config::standard();
        let encoded = bincode::encode_to_vec(self, config)?;
        fs::write(cache_path, encoded)?;
        Ok(())
    }

    pub fn save_cache(&self) -> Result<()> {
        let config = config::standard();
        let encoded = bincode::encode_to_vec(self, config)?;
        let path = self.cache_path();
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, encoded)?;
        Ok(())
    }

    pub fn load_cache(path: &Path) -> Result<Self> {
        let config = config::standard();
        let bytes = fs::read(path)?;
        let (decoded, _len): (Self, usize) = bincode::decode_from_slice(&bytes[..], config)?;
        Ok(decoded)
    }

    pub fn activation_path(&self) -> PathBuf {
        self.binaries.clone()
    }

    pub fn requirements(&self) -> PathBuf {
        PathBuf::from_str(&self.name)
            .unwrap()
            .join(&self.binaries)
            .join(PathBuf::from_str("python").unwrap())
    }
}

impl VenvUi {
    pub fn new(venv: Arc<Venv>, last_modified: DateTime<Local>) -> Self {
        Self {
            scroll_state: ScrollbarState::new(venv.packages.len()),
            list_state: ListState::default().with_selected(Some(0)),
            last_modified,
            venv,
        }
    }
}

impl VenvListUi {
    pub fn new(venvs: Vec<Arc<Venv>>) -> Self {
        let venvs_ui: Vec<VenvUi> = venvs
            .into_iter()
            .map(|v| {
                let last_modified = v
                    .packages
                    .iter()
                    .map(|pkg| pkg.last_modified)
                    .max()
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let date: DateTime<Local> = last_modified.into();
                VenvUi::new(v, date)
            })
            .collect();
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            scroll_state: ScrollbarState::new(venvs_ui.len()),
            venvs: venvs_ui,
        }
    }
}
