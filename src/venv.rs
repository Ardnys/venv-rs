use std::{
    fs, io,
    path::{Path, PathBuf},
};

use ratatui::widgets::ListState;

// TODO: might add more details later
#[derive(Debug, Clone)]
pub struct Venv {
    pub name: String,
    // TODO: packages may be more complex later
    pub packages: Vec<String>,
    pub state: ListState,
}

impl Venv {
    pub fn new(name: String, packages: Vec<String>) -> Self {
        Self {
            name,
            packages,
            state: ListState::default(),
        }
    }

    fn parse_config_file_contents(contents: String) -> String {
        contents
            .lines()
            .find(|l| l.trim_start().starts_with("version"))
            .expect("Could not find version in config")
            .split('=')
            .nth(1)
            .map(|v| v.trim().to_string())
            .unwrap()
    }
    fn get_python_dir(lib_dir: PathBuf) -> io::Result<Option<PathBuf>> {
        let mut entries = fs::read_dir(lib_dir)?
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir());

        Ok(entries.next().map(|e| e.path()))
    }
    fn get_dist_info_packages(site_packages: PathBuf) -> io::Result<()> {
        for package in fs::read_dir(site_packages)? {
            let dir_path = package?.path();
            let dir_name = dir_path.to_str().unwrap();
            if dir_name.ends_with("dist-info") {
                println!("{dir_name}");
            }
        }
        Ok(())
    }
    // expects dir to be a virtual environment
    pub fn parse_from_dir(dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            println!("Reading dir: {}", dir.to_str().unwrap());
            let pyvevnv_cfg_file = dir.join("pyvenv.cfg");
            let cfg_contents = fs::read_to_string(&pyvevnv_cfg_file)?;
            let version = Venv::parse_config_file_contents(cfg_contents);
            println!("Python version: {version}");

            #[cfg(target_os = "windows")]
            let binaries = dir.join("Scripts");

            #[cfg(target_os = "linux")]
            let binaries = dir.join("bin");
            println!("Binary directory: {}", binaries.to_str().unwrap());

            let lib_dir = dir.join("lib");
            if let Some(python_dir) = Venv::get_python_dir(lib_dir)? {
                let site_packages = python_dir.join("site-packages");
                Venv::get_dist_info_packages(site_packages)?;
            }

            // returns an iterator but that directory probably has only one folder. i
            // don't care about the iterator so the first thing in the iterator is fine
        } else {
            println!("{} not dir", dir.to_str().unwrap());
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VenvList {
    // TODO: rename it something else
    pub venvs: Vec<Venv>,
    pub state: ListState,
}

impl FromIterator<(&'static str, Vec<&'static str>)> for VenvList {
    fn from_iter<T: IntoIterator<Item = (&'static str, Vec<&'static str>)>>(iter: T) -> Self {
        let items = iter
            .into_iter()
            .map(|(name, packages)| {
                Venv::new(
                    name.to_string(),
                    packages.iter().map(|package| package.to_string()).collect(),
                )
            })
            .collect();
        Self {
            venvs: items,
            state: ListState::default(),
        }
    }
}
