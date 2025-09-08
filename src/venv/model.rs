use std::{
    collections::BTreeMap,
    fs::{self},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, RwLock, mpsc::Sender},
    thread::{self, JoinHandle},
    time::SystemTime,
};

use bincode::{Decode, Encode, config};
use chrono::{DateTime, Local};
use color_eyre::Result;
use color_eyre::eyre;
use dirs::cache_dir;
use ratatui::widgets::{ListState, ScrollbarState};

use crate::{app::SyncMsg, venv::metadata::Metadata};

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

fn to_cache_path(venv_path: &Path, cache_dir: &Path) -> Option<PathBuf> {
    let fname = venv_path
        .file_name()
        .expect("Could not get the filename")
        .to_str()
        .unwrap();

    let cached_fname = format!("{fname}.bin");
    let cached_file = cache_dir.join(cached_fname);

    Some(cached_file)
}

#[derive(Debug)]
pub struct VenvManager {
    cache: BTreeMap<PathBuf, Arc<Venv>>,
    cache_path: PathBuf,
}

impl Default for VenvManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VenvManager {
    pub fn new() -> Self {
        let cache_path = cache_dir()
            .expect("Could not get cache dir")
            .join("venv_rs");

        fs::create_dir_all(&cache_path).expect("Failed to create them dirs");

        Self {
            cache: BTreeMap::new(),
            cache_path,
        }
    }

    pub fn threaded_sync(vm_arc: Arc<RwLock<Self>>, sender: Sender<SyncMsg>) -> JoinHandle<()> {
        thread::spawn(move || {
            let _ = sender.send(SyncMsg::Started);

            // get the entires with a read lock
            let snapshot: Vec<(PathBuf, Arc<Venv>)> = {
                let vm_r = vm_arc.read().expect("rwlock poisoned");
                vm_r.cache
                    .iter()
                    .map(|(p, v)| (p.clone(), Arc::clone(v)))
                    .collect()
            };

            // check for stale venvs off lock
            for (path, cached_venv) in snapshot {
                let _ = sender.send(SyncMsg::Progress {
                    venv: cached_venv.name.clone(),
                });

                let cached_most_recent = cached_venv
                    .packages
                    .iter()
                    .map(|pkg| pkg.last_modified)
                    .max()
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                // compute most recent on-disk dist-info mtime (expensive, but off-lock)
                let most_recent_on_disk =
                    match VenvParser::new(path.clone()).recent_dist_info_modification() {
                        Ok(t) => t,
                        Err(e) => {
                            // If we can't stat the directory, report an error and skip updating
                            let _ = sender.send(SyncMsg::Error(format!(
                                "Failed to stat dist-info for {}: {}",
                                path.display(),
                                e
                            )));
                            // continue to next venv
                            continue;
                        }
                    };
                // decide if stale
                if cached_most_recent < most_recent_on_disk {
                    // expensive parse (off-lock)
                    match Venv::from_path(&path) {
                        Ok(new_venv) => {
                            // short write lock to update the cache atomically
                            {
                                let mut vm_w = vm_arc.write().expect("rwlock poisoned");
                                vm_w.cache.insert(path.clone(), Arc::new(new_venv));
                            }
                            let _ = sender.send(SyncMsg::VenvUpdated(cached_venv.name.clone()));
                        }
                        Err(e) => {
                            let _ = sender.send(SyncMsg::Error(format!(
                                "Failed to parse {}: {}",
                                path.display(),
                                e
                            )));
                        }
                    }
                } else {
                    // no change — optionally still notify so UI can clear spinner
                    let _ = sender.send(SyncMsg::VenvUpdated(cached_venv.name.clone()));
                }
            }
            // optionally save cache at the end under read lock (assuming save reads cache)
            {
                let vm_r = vm_arc.read().expect("rwlock poisoned");
                if let Err(e) = vm_r.save_cache() {
                    let _ = sender.send(SyncMsg::Error(format!("Failed to save cache: {}", e)));
                }
            }
            let _ = sender.send(SyncMsg::Finished);
        })
    }

    pub fn venvs_from_cache(&self) -> Result<Vec<Venv>> {
        fs::read_dir(&self.cache_path)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .map(|path| Venv::load_cache(&path))
            .collect()
    }

    pub fn load_cache(&mut self) -> Result<()> {
        if let Ok(venvs) = self.venvs_from_cache() {
            self.cache = venvs
                .into_iter()
                .map(|v| (v.path.clone(), Arc::new(v)))
                .collect();
        }
        Ok(())
    }

    pub fn save_cache(&self) -> Result<()> {
        for v in self.cache.values() {
            if let Some(cache_path) = to_cache_path(&v.path, &self.cache_path) {
                v.save_cache_to(&cache_path)?;
            }
        }

        Ok(())
    }

    pub fn get(&mut self, p: &Path) -> Result<Arc<Venv>> {
        // println!("Getting {}", p.to_string_lossy().yellow().italic());
        let kee = self.cache.contains_key(p);
        // let stale = self.is_venv_stale(p);
        // println!("Contains: {kee}, Stale: {stale}");
        if !kee {
            let venv = Venv::from_path(p)?;
            self.cache.insert(p.to_path_buf(), venv.into());
        }
        Ok(self.cache.get(p).unwrap().clone())
    }

    pub fn reload_venv(&mut self, p: &Path) -> Result<()> {
        let venv = Venv::from_path(p)?;
        self.cache.insert(p.to_path_buf(), venv.into());
        Ok(())
    }

    pub fn sync_cache(&mut self) {
        // Collect keys to update to avoid mutable/immutable borrow conflict
        let keys_to_update: Vec<PathBuf> = self
            .cache
            .keys()
            .filter(|k| self.is_venv_stale(k))
            .cloned()
            .collect();

        for k in keys_to_update {
            let venv = Venv::from_path(&k).unwrap();
            self.cache.insert(k, venv.into());
        }
    }

    pub fn is_venv_stale(&self, p: &Path) -> bool {
        if !self.cache.contains_key(p) {
            return true;
        }
        let v = self.cache.get(p).unwrap();

        let last_modified = v
            .packages
            .iter()
            .map(|p| p.last_modified)
            .max()
            .unwrap_or(SystemTime::UNIX_EPOCH);

        // get the package files to check the versions, if they changed
        let parser = VenvParser::new(p.to_path_buf());
        let most_recent_update = parser
            .recent_dist_info_modification()
            .expect("Failed to get recent update");

        last_modified < most_recent_update
    }

    pub fn get_venvs(&self) -> Vec<Arc<Venv>> {
        self.cache.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::PathBuf};

    use claims::assert_ok;
    use tempfile::tempdir;

    use crate::venv::model::VenvManager;

    fn prepare() -> VenvManager {
        let cache_dir = tempdir().unwrap().path().join("venv_rs");
        fs::create_dir_all(&cache_dir).expect("Failed to create them dirs");
        let mut vman = VenvManager {
            cache: BTreeMap::new(),
            cache_path: cache_dir.to_path_buf(),
        };
        let venv_path = PathBuf::from(".venv/testenv");
        if let Ok(yes) = fs::exists(&venv_path) {
            assert!(yes);
        }
        let _ = vman.get(&venv_path).expect("Error while parsing venv");

        vman
    }

    #[test]
    fn saved_cache() {
        let vm = prepare();

        let res = vm.save_cache();
        assert_ok!(res);

        let mut it = vm
            .cache
            .values()
            .zip(fs::read_dir(vm.cache_path).expect("Failed to read cache dir"));

        let (venv, cached_file) = it.next().unwrap();
        let cached_file = cached_file.expect("Error while getting cached file").path();
        let fname = cached_file.file_stem().unwrap();
        let cached_file_name = fname.to_str().unwrap();
        assert_eq!(venv.name, cached_file_name);
    }

    #[test]
    fn load_cache() {
        let mut vm = prepare();
        let _ = vm.save_cache();
        let res = vm.load_cache();
        assert_ok!(res);
    }

    #[test]
    fn cache_not_stale() {
        let vm = prepare();
        let test_venv_path = PathBuf::from(".venv/testenv");
        let res = vm.is_venv_stale(&test_venv_path);
        assert!(!res);
    }
}
