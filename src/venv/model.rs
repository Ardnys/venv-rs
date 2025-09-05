use std::{
    collections::HashMap,
    fs::{self},
    path::{Path, PathBuf},
    str::FromStr,
};

use bincode::{Decode, Encode, config};
use color_eyre::Result;
use color_eyre::eyre;
use dirs::{cache_dir, config_dir};
use ratatui::widgets::{ListState, ScrollbarState};

use crate::venv::{metadata::Metadata, parser::parse_from_dir};

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
    pub venv: Venv,
    pub list_state: ListState,
    pub scroll_state: ScrollbarState,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct VenvListUi {
    pub venvs: Vec<VenvUi>,
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
            packages,
            num_dist_info_packages,
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

    pub fn save_cache(&self) -> Result<()> {
        let config = config::standard();
        let encoded = bincode::encode_to_vec(&self, config)?;
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
    pub fn new(venv: Venv) -> Self {
        Self {
            scroll_state: ScrollbarState::new(venv.packages.len()),
            venv: venv,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl VenvListUi {
    pub fn new(venvs: Vec<Venv>) -> Self {
        let venvs_ui: Vec<VenvUi> = venvs.into_iter().map(|v| VenvUi::new(v)).collect();
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            scroll_state: ScrollbarState::new(venvs_ui.len()),
            venvs: venvs_ui,
        }
    }
}

pub struct VenvManager {
    cache: HashMap<PathBuf, Venv>,
    cache_path: PathBuf,
}

impl VenvManager {
    pub fn new() -> Self {
        let cache_path = cache_dir()
            .expect("Could not get cache dir")
            .join("venv_rs");

        Self {
            cache: HashMap::new(),
            cache_path,
        }
    }

    pub fn load_cache(&mut self) -> Result<()> {
        if let Ok(venvs) = Venv::from_cache(&self.cache_path) {
            self.cache = venvs.into_iter().map(|v| (v.path.clone(), v)).collect();
        }
        Ok(())
    }

    pub fn get(&mut self, p: &Path) -> Result<&Venv> {
        if !self.cache.contains_key(p) {
            let venv = Venv::from_path(p)?;
            self.cache.insert(p.to_path_buf(), venv);
        }
        Ok(&self.cache.get(p).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs, path::PathBuf};

    use tempfile::tempdir;

    use crate::venv::model::VenvManager;

    fn prepare() -> VenvManager {
        let cache_dir = tempdir().unwrap();
        let mut vman = VenvManager {
            cache: HashMap::new(),
            cache_path: cache_dir.path().to_path_buf(),
        };
        let venv_path = PathBuf::from(".venv/testenv");
        if let Ok(yes) = fs::exists(&venv_path) {
            assert!(yes);
        }
        let _ = vman.get(&venv_path).expect("Error while parsing venv");

        vman
    }

    // #[test]
    // fn empty_cache() {
    //     let cache_dir_empty = fs::read_dir(cache_dir).unwrap().next().is_none();
    //     assert!(cache_dir_empty);
    // }
}
